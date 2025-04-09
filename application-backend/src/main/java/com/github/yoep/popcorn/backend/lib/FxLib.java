package com.github.yoep.popcorn.backend.lib;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.FxMessage;
import lombok.extern.slf4j.Slf4j;

import java.io.*;
import java.net.StandardProtocolFamily;
import java.net.UnixDomainSocketAddress;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.channels.Channels;
import java.nio.channels.ServerSocketChannel;
import java.nio.channels.SocketChannel;
import java.nio.file.Files;
import java.nio.file.Paths;
import java.util.ArrayList;
import java.util.Objects;
import java.util.UUID;

import static java.util.Arrays.asList;

@Slf4j
public class FxLib implements Closeable {
    BufferedInputStream reader;
    BufferedOutputStream writer;
    Process process;
    RandomAccessFile namedPipe;
    ServerSocketChannel unixSocket;
    SocketChannel channel;

    public FxLib(String[] args) {
        try {
            if (isWindows()) {
                createNamedPipe(args);
            } else {
                createDomainSocket(args);
            }
        } catch (Exception e) {
            throw new RuntimeException(e);
        }
    }

    @Override
    public void close() throws IOException {
        if (namedPipe != null) {
            namedPipe.close();
        }
        if (unixSocket != null) {
            unixSocket.close();
        }
        if (process != null && process.isAlive()) {
            process.destroy();
        }
    }

    public FxMessage receive() {
        try {
            var length_bytes = reader.readNBytes(4);
            if (length_bytes.length != 4) {
                throw new FxChannelException("Channel received EOF");
            }

            var length = fromBigEndian(length_bytes);
            var message_bytes = reader.readNBytes(length);

            return FxMessage.parseFrom(message_bytes);
        } catch (IOException e) {
            log.error("Failed to read IPC message", e);
            throw new FxChannelException(e.getMessage(), e);
        }
    }

    public void send(FxMessage message) {
        Objects.requireNonNull(message, "message cannot be null");
        var message_bytes = message.toByteArray();
        var buffer = ByteBuffer.allocate(4 + message_bytes.length);

        // write the length of the message as BigEndian in the first 4 bytes
        buffer.putInt(message_bytes.length);
        // append the serialized message
        buffer.put(message_bytes);

        try {
            writer.write(buffer.array());
            writer.flush();
        } catch (IOException e) {
            log.error("Failed to write IPC message", e);
        }
    }

    private void createNamedPipe(String[] args) throws Exception {
        var socketPath = "libfx";

        namedPipe = new RandomAccessFile(String.format("\\\\.\\pipe\\%s", socketPath), "rw");
        process = new ProcessBuilder("libfx.exe", socketPath)
                .inheritIO()
                .start();

        var fd = namedPipe.getFD();
        reader = new BufferedInputStream(new FileInputStream(fd));
        writer = new BufferedOutputStream(new FileOutputStream(fd));
    }

    private void createDomainSocket(String[] args) throws Exception {
        var socketPath = String.format("/tmp/libfx.%s.sock", UUID.randomUUID().toString().replace("-", ""));
        Files.deleteIfExists(Paths.get(socketPath));

        var address = UnixDomainSocketAddress.of(socketPath);
        unixSocket = ServerSocketChannel.open(StandardProtocolFamily.UNIX);
        unixSocket.bind(address);

        var processCommand = new ArrayList<>(asList("libfx", socketPath));
        processCommand.addAll(asList(args));
        process = new ProcessBuilder(processCommand)
                .inheritIO()
                .start();

        channel = unixSocket.accept();
        unixSocket.configureBlocking(false);

        reader = new BufferedInputStream(Channels.newInputStream(channel));
        writer = new BufferedOutputStream(Channels.newOutputStream(channel));
    }

    private static boolean isWindows() {
        return System.getProperty("os.name").toLowerCase().contains("win");
    }

    /**
     * Read an int value from the given BigEndian byte array.
     *
     * @param bytes The byte array to read from.
     * @return Returns the int value read from the byte array.
     */
    public static int fromBigEndian(byte[] bytes) {
        var buffer = ByteBuffer.wrap(bytes);
        buffer.order(ByteOrder.BIG_ENDIAN);
        return buffer.getInt();
    }
}
