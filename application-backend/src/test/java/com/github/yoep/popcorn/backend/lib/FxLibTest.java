package com.github.yoep.popcorn.backend.lib;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.FxMessage;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.GetActivePlayerRequest;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.GetMediaDetailsRequest;
import lombok.extern.slf4j.Slf4j;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.net.StandardProtocolFamily;
import java.net.UnixDomainSocketAddress;
import java.nio.ByteBuffer;
import java.nio.channels.SocketChannel;
import java.time.Duration;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.*;

@Slf4j
class FxLibTest {

    @Test
    void testNewInstance() throws IOException {
        var socketChannel = new AtomicReference<SocketChannel>();

        try (var fxLib = new FxLib(new String[0]) {
            @Override
            Process launchLibProcess(String socketPath, String libraryExecutable, String[] args) {
                return FxLibTest.launchLibProcess(socketPath, socketChannel);
            }
        }) {
            assertTimeout(Duration.ofSeconds(1), () -> socketChannel.get() != null, "expected a socket channel to have been created");
            assertNotNull(fxLib.reader, "expected a reader to have been created");
            assertNotNull(fxLib.writer, "expected a writer to have been created");
        }
    }

    @Test
    void testReceive() throws IOException, ExecutionException, InterruptedException, TimeoutException {
        var socketChannel = new AtomicReference<SocketChannel>();
        var messageType = FxChannel.typeFrom(GetMediaDetailsRequest.class);
        var message = FxMessage.newBuilder()
                .setType(messageType)
                .setSequenceId(1)
                .build();
        var messageBytes = message.toByteArray();
        var buffer = ByteBuffer.allocate(4 + messageBytes.length);
        buffer.putInt(messageBytes.length);
        buffer.put(messageBytes);

        try (var fxLib = new FxLib(new String[0]) {
            @Override
            Process launchLibProcess(String socketPath, String libraryExecutable, String[] args) {
                return FxLibTest.launchLibProcess(socketPath, socketChannel);
            }
        }) {
            assertTimeout(Duration.ofSeconds(1), () -> socketChannel.get() != null, "expected a socket channel to have been created");

            var future = CompletableFuture.supplyAsync(fxLib::receive);
            buffer.flip();
            socketChannel.get().write(buffer);
            var result = future.get(2, TimeUnit.SECONDS);

            assertNotNull(result, "expected to receive a message");
            assertEquals(messageType, result.getType());
            assertEquals(1, result.getSequenceId());
        }
    }

    @Test
    void testSend() throws IOException {
        var socketChannel = new AtomicReference<SocketChannel>();
        var messageType = FxChannel.typeFrom(GetActivePlayerRequest.class);
        var message = FxMessage.newBuilder()
                .setType(messageType)
                .setSequenceId(2)
                .build();

        try (var fxLib = new FxLib(new String[0]) {
            @Override
            Process launchLibProcess(String socketPath, String libraryExecutable, String[] args) {
                return FxLibTest.launchLibProcess(socketPath, socketChannel);
            }
        }) {
            assertTimeout(Duration.ofSeconds(1), () -> socketChannel.get() != null, "expected a socket channel to have been created");

            fxLib.send(message);

            var result = readMessageFromChannel(socketChannel.get());
            assertNotNull(result, "expected to receive a message");
            assertEquals(messageType, result.getType());
            assertEquals(2, result.getSequenceId());
        }
    }

    private static Process launchLibProcess(String socketPath, AtomicReference<SocketChannel> socketChannel) {
        var address = UnixDomainSocketAddress.of(socketPath);
        new Thread(() -> {
            try {
                var clientChannel = SocketChannel.open(StandardProtocolFamily.UNIX);
                clientChannel.connect(address);
                socketChannel.set(clientChannel);
            } catch (IOException ex) {
                log.error("Socket channel error", ex);
            }
        }, "FxLibSocketChannel").start();
        return null;
    }

    private static FxMessage readMessageFromChannel(SocketChannel socketChannel) throws IOException {
        var lengthBuffer = ByteBuffer.allocate(4);

        socketChannel.read(lengthBuffer);
        var length = FxLib.fromBigEndian(lengthBuffer.array());
        var messageBuffer = ByteBuffer.allocate(length);
        socketChannel.read(messageBuffer);

        return FxMessage.parseFrom(messageBuffer.array());
    }
}