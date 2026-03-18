package com.github.yoep.popcorn.backend.playlists;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Playlist;

public interface PlaylistManagerListener {
    /**
     * Invoked when a new playlist has been set.
     */
    void onPlaylistChanged();

    void onPlayNextChanged(PlayNext next);

    void onPlayNextIn(int seconds);

    void onPlayNextInAborted();

    void onStateChanged(Playlist.State state);
}
