package com.github.yoep.popcorn.backend.lib;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.FxMessage;
import com.google.protobuf.ByteString;
import com.google.protobuf.InvalidProtocolBufferException;
import com.google.protobuf.MessageLite;
import com.google.protobuf.Parser;
import lombok.extern.slf4j.Slf4j;

import java.io.Closeable;
import java.io.IOException;
import java.util.Objects;
import java.util.Optional;
import java.util.Queue;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
public class FxChannel implements Closeable {
    public static final AtomicReference<FxChannel> INSTANCE = new AtomicReference<>();
    private final FxLib fxLib;

    final AtomicBoolean running = new AtomicBoolean(true);
    final AtomicInteger sequenceId = new AtomicInteger();
    final Queue<PendingRequest> requests = new ConcurrentLinkedQueue<>();

    Thread readerThread;

    public FxChannel(FxLib fxLib) {
        Objects.requireNonNull(fxLib, "fxLib cannot be null");
        this.fxLib = fxLib;
        INSTANCE.set(this);
        init();
    }

    public <T extends MessageLite> CompletableFuture<T> get(FxMessage.MessageType type, Parser<T> parser) {
        var id = sendInternally(ByteString.empty(), type);
        var future = new CompletableFuture<T>();

        requests.add(new PendingRequest<>(id, parser, future));

        return future;
    }

    public void send(MessageLite message) {
        var bytes = message.toByteString();
        var type = typeFrom(message);

        sendInternally(bytes, type);
    }

    @Override
    public void close() throws IOException {
        running.set(false);
        readerThread.interrupt();
        sendInternally(ByteString.empty(), FxMessage.MessageType.TERMINATE);
        fxLib.close();
    }

    private void init() {
        readerThread = new Thread(() -> {
            while (running.get()) {
                var message = fxLib.receive();
                log.trace("FX channel received {}", message.getType());

                if (message.hasReplyId()) {
                    Optional.of(message.getReplyId())
                            .flatMap(id -> requests.stream()
                                    .filter(r -> r.sequenceId == id)
                                    .findFirst())
                            .ifPresent(request -> {
                                log.debug("IPC channel received response for request {}", request.sequenceId);
                                try {
                                    var data = request.parser.parseFrom(message.getPayload());
                                    request.future.complete(data);
                                } catch (InvalidProtocolBufferException e) {
                                    log.error("FX channel failed to parse message", e);
                                } finally {
                                    requests.remove(request);
                                }
                            });
                } else {
                    log.warn("FX channel received message without reply ID for type {}", message.getType());
                }
            }
        }, "FxChannel");
        readerThread.start();
    }

    private int sendInternally(ByteString bytes, FxMessage.MessageType type) {
        log.trace("FX channel is sending {}", type);
        var sequenceId = this.sequenceId.incrementAndGet();

        fxLib.send(FxMessage.newBuilder()
                .setType(type)
                .setSequenceId(sequenceId)
                .setPayload(bytes)
                .build());

        return sequenceId;
    }

    static FxMessage.MessageType typeFrom(MessageLite message) {
        return switch (message.getClass().getName()) {
            case "com.github.yoep.popcorn.backend.lib.ipc.protobuf.Log" -> FxMessage.MessageType.LOG_MESSAGE;
            default -> throw new IllegalArgumentException("Unknown message type: " + message.getClass());
        };
    }

    public record PendingRequest<T extends MessageLite>(
            int sequenceId,
            Parser<T> parser,
            CompletableFuture<T> future
    ) {
    }
}
