package com.github.yoep.popcorn.backend.lib;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationTerminationRequest;
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
import java.util.concurrent.Executor;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
public class FxChannel implements Closeable {
    public static final AtomicReference<FxChannel> INSTANCE = new AtomicReference<>();
    private final FxLib fxLib;
    private final Executor executor;

    final AtomicBoolean running = new AtomicBoolean(true);
    final AtomicInteger sequenceId = new AtomicInteger();
    final Queue<PendingRequest> requests = new ConcurrentLinkedQueue<>();
    final Queue<Subscription> subscriptions = new ConcurrentLinkedQueue<>();
    final Queue<ReplySubscription> replySubscriptions = new ConcurrentLinkedQueue<>();

    Thread readerThread;

    public FxChannel(FxLib fxLib, Executor executor) {
        Objects.requireNonNull(fxLib, "fxLib cannot be null");
        this.fxLib = fxLib;
        this.executor = executor;
        INSTANCE.set(this);
        init();
    }

    /**
     * Get data for the given type from the fxChannel.
     *
     * @param request The request message to send.
     * @param parser  The parser to use for parsing the message.
     * @param <T>     The response data type.
     * @return Returns a future that completes when the data is received.
     */
    public <T extends MessageLite> CompletableFuture<T> send(
            MessageLite request,
            Parser<T> parser
    ) {
        return sendInternally(
                request.toByteString(),
                typeFrom(request),
                parser
        );
    }

    /**
     * Send the given message to the fxChannel.
     *
     * @param message The message to send.
     */
    public void send(MessageLite message) {
        var bytes = message.toByteString();
        var type = typeFrom(message);

        sendBytes(bytes, type, null);
    }

    /**
     * Send the given message to the fxChannel.
     *
     * @param message   The message to send.
     * @param replyToId The original request ID to which is being responded.
     */
    public void send(MessageLite message, Integer replyToId) {
        var bytes = message.toByteString();
        var type = typeFrom(message);

        sendBytes(bytes, type, replyToId);
    }

    public <T extends MessageLite> void subscribe(
            String type,
            Parser<T> parser,
            FxCallback<T> callback
    ) {
        subscriptions.add(new Subscription<>(type, parser, callback));
    }

    public <T extends MessageLite> void subscribe_response(
            String type,
            Parser<T> parser,
            FxReplyCallback<T> callback
    ) {
        replySubscriptions.add(new ReplySubscription<>(type, parser, callback));
    }

    @Override
    public void close() throws IOException {
        send(ApplicationTerminationRequest.getDefaultInstance());
        running.set(false);
        readerThread.interrupt();
        fxLib.close();
    }

    public static String typeFrom(MessageLite message) {
        return typeFrom(message.getClass());
    }

    public static String typeFrom(Class<? extends MessageLite> message) {
        return message.getSimpleName();
    }

    private void init() {
        readerThread = new Thread(() -> {
            while (running.get()) {
                try {
                    var message = fxLib.receive();
                    log.trace("IPC channel received {}", message.getType());
                    executor.execute(() -> processMessage(message));
                } catch (FxChannelException ex) {
                    log.error("IPC channel receiver encountered an error, {}", ex.getMessage(), ex);
                    running.set(false);
                } catch (Exception ex) {
                    log.error("IPC channel receiver encountered an error, {}", ex.getMessage(), ex);
                }
            }

            log.debug("IPC channel reader ended");
        }, "FxChannel");
        readerThread.start();
    }

    private <T extends MessageLite> CompletableFuture<T> sendInternally(
            ByteString request,
            String type,
            Parser<T> parser
    ) {
        var sequenceId = this.sequenceId.incrementAndGet();
        var future = (CompletableFuture<T>) null;

        if (parser != null) {
            future = new CompletableFuture<T>();
            requests.add(new PendingRequest<>(sequenceId, parser, future));
        }

        sendBytes(request, type, sequenceId, null);

        return future;
    }

    private void sendBytes(ByteString bytes, String type, Integer replyToId) {
        var sequenceId = this.sequenceId.incrementAndGet();
        sendBytes(bytes, type, sequenceId, replyToId);
    }

    private void sendBytes(ByteString bytes, String type, int sequenceId, Integer replyToId) {
        log.trace("IPC channel is sending {}", type);
        var messageBuilder = FxMessage.newBuilder()
                .setType(type)
                .setSequenceId(sequenceId)
                .setPayload(bytes);

        Optional.ofNullable(replyToId)
                .ifPresent(messageBuilder::setReplyTo);

        fxLib.send(messageBuilder.build());
    }

    private void processMessage(FxMessage message) {
        if (message.hasReplyTo()) {
            Optional.of(message.getReplyTo())
                    .flatMap(id -> requests.stream()
                            .filter(r -> r.sequenceId == id)
                            .findFirst())
                    .ifPresent(request -> {
                        log.debug("IPC channel received response for request {}", request.sequenceId);
                        try {
                            var data = request.parser.parseFrom(message.getPayload());

                            if (!request.future.isCancelled()) {
                                request.future.complete(data);
                            } else {
                                log.trace("IPC channel reply future has already been cancelled for {}", request.sequenceId);
                            }
                        } catch (InvalidProtocolBufferException e) {
                            log.error("IPC channel failed to parse message", e);
                        } finally {
                            requests.remove(request);
                        }
                    });
        } else {
            Optional.ofNullable(message.getType()).ifPresent(type -> replySubscriptions.stream()
                    .filter(s -> Objects.equals(s.type, type))
                    .forEach(subscription -> {
                        try {
                            var data = subscription.parser.parseFrom(message.getPayload());
                            subscription.callback.callback(message.getSequenceId(), data);
                        } catch (InvalidProtocolBufferException e) {
                            log.error("IPC channel failed to parse message", e);
                        }
                    }));
            Optional.ofNullable(message.getType()).ifPresent(type -> subscriptions.stream()
                    .filter(s -> Objects.equals(s.type, type))
                    .forEach(subscription -> {
                        try {
                            var data = subscription.parser.parseFrom(message.getPayload());
                            subscription.callback.callback((MessageLite) data);
                        } catch (InvalidProtocolBufferException e) {
                            log.error("IPC channel failed to parse message", e);
                        }
                    }));

        }
    }

    public record PendingRequest<T extends MessageLite>(
            int sequenceId,
            Parser<T> parser,
            CompletableFuture<T> future
    ) {
    }

    public record Subscription<T extends MessageLite>(
            String type,
            Parser<T> parser,
            FxCallback<T> callback
    ) {
    }

    public record ReplySubscription<T extends MessageLite>(
            String type,
            Parser<T> parser,
            FxReplyCallback<T> callback
    ) {
    }
}
