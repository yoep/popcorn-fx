package com.github.yoep.popcorn.backend.lib;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.FxMessage;
import lombok.extern.slf4j.Slf4j;

import java.io.*;
import java.net.ServerSocket;
import java.net.Socket;
import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.util.ArrayList;
import java.util.Objects;

import static java.util.Arrays.asList;

@Slf4j
public class FxLib implements Closeable {
    InputStream reader;
    OutputStream writer;
    Process process;
    ServerSocket serverSocket;
    Socket socket;

    public FxLib(String[] args) {
        try {
            createSocket(args);
        } catch (Exception e) {
            throw new RuntimeException(e);
        }
    }

    @Override
    public void close() throws IOException {
        // close the IO streams before closing any underlying channels
        reader.close();
        writer.close();

        // close the underlying sockets
        socket.close();
        serverSocket.close();

        if (process != null && process.isAlive()) {
            process.destroy();
        }
    }

    /**
     * Receive an incoming message from the library subprocess.
     *
     * @return Returns the received message.
     * @throws FxLibException Throws an exception when the connection closed unexpectedly or an IO error occurred.
     */
    public FxMessage receive() {
        try {
            var length_bytes = reader.readNBytes(4);
            if (length_bytes.length != 4) {
                throw new FxLibException("Channel received EOF");
            }

            var length = fromBigEndian(length_bytes);
            var message_bytes = reader.readNBytes(length);

            return FxMessage.parseFrom(message_bytes);
        } catch (IOException e) {
            log.error("Failed to read IPC message", e);
            throw new FxLibException(e.getMessage(), e);
        }
    }

    /**
     * Send the given message to the library subprocess.
     *
     * @param message The message to send (required).
     */
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

    /**
     * Launch the library subprocess for this lib instance.
     *
     * @param socketPath        The socket path to which the lib needs to connect to.
     * @param libraryExecutable The executable filename of the library.
     * @param args              The library arguments.
     * @return Returns the library process.
     * @throws IOException Throws an IO exception when the library couldn't be started.
     */
    Process launchLibProcess(String socketPath, String libraryExecutable, String[] args) throws IOException {
        var processCommand = new ArrayList<>(asList(libraryExecutable, socketPath));
        processCommand.addAll(asList(args));
        return new ProcessBuilder(processCommand)
                .inheritIO()
                .start();
    }

    private void createSocket(String[] args) {
        try (var serverSocket = new ServerSocket(0)) {
            var port = serverSocket.getLocalPort();
            var libraryExecutable = isWindows() ? "libfx.exe" : "libfx";

            process = launchLibProcess(String.valueOf(port), libraryExecutable, args);

            var socket = serverSocket.accept();

            reader = new BufferedInputStream(socket.getInputStream());
            writer = new BufferedOutputStream(socket.getOutputStream());

            this.socket = socket;
            this.serverSocket = serverSocket;
        } catch (IOException ex) {
            throw new FxLibException(String.format("Failed to start IPC process, %s", ex.getMessage()), ex);
        }
    }

    static boolean isWindows() {
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
