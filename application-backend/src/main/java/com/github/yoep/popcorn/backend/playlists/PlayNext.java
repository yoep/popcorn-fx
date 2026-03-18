package com.github.yoep.popcorn.backend.playlists;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Playlist;

public sealed interface PlayNext {
    record Next(Playlist.Item item) implements PlayNext {}

    record End() implements PlayNext {}

    /**
     * Create a PlayNext instance from the given protobuf.
     * @param proto The protobuf to create the PlayNext instance from.
     * @return Returns the PlayNext instance.
     */
    static PlayNext from(com.github.yoep.popcorn.backend.lib.ipc.protobuf.PlayNext proto) {
        switch (proto.getType()) {
            case NEXT -> {
                return new Next(proto.getNext().getItem());
            }
            case END -> {
                return new End();
            }
            default -> throw new IllegalArgumentException("invalid play next type");
        }
    }
}
