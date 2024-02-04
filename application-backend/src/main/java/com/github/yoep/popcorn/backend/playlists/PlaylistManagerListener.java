package com.github.yoep.popcorn.backend.playlists;

public interface PlaylistManagerListener {
    void onPlaylistChanged();

    void onPlayingIn(Long playingIn, PlaylistItem item);

    void onStateChanged(PlaylistState state);
}
