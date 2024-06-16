package com.github.yoep.popcorn.backend.playlists;

import com.github.yoep.popcorn.backend.playlists.model.PlaylistItem;

public interface PlaylistManagerListener {
    void onPlaylistChanged();

    void onPlayingIn(Long playingIn, PlaylistItem item);

    void onStateChanged(PlaylistState state);
}
