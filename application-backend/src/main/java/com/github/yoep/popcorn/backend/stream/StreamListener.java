package com.github.yoep.popcorn.backend.stream;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Stream;

public interface StreamListener {
    /**
     * Invoked when the stream state has changed.
     * @param state The new state of the stream.
     */
    void onStateChanged(Stream.StreamState state);

    /**
     * Invoked when the stream stats have changed.
     * @param stats The new stats of the stream.
     */
    void onStatsChanged(Stream.StreamStats stats);
}
