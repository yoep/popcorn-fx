package com.github.yoep.popcorn.backend.stream;

public interface IStreamServer {
    /**
     * Add a listener to the stream server.
     *
     * @param filename The filename of the stream.
     * @param listener The listener to add.
     */
    void addListener(String filename, StreamListener listener);

    /**
     * Remove the listener from the stream server.
     *
     * @param filename The filename of the stream.
     * @param listener The listener to remove.
     */
    void removeListener(String filename, StreamListener listener);
}
