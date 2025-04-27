package com.github.yoep.popcorn.backend.playlists;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Playlist;

public interface PlaylistManagerListener {
    void onPlaylistChanged();

    void onPlayingIn(Long playingIn, Playlist.Item item);

    void onStateChanged(Playlist.State state);
}
