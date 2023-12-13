package com.github.yoep.popcorn.backend.playlists;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;

public class PlaylistManager {
    private final FxLib fxLib;
    private final PopcornFx instance;

    public PlaylistManager(FxLib fxLib, PopcornFx instance) {
        this.fxLib = fxLib;
        this.instance = instance;
    }

    public void play(Playlist playlist) {

    }

    public void play(PlaylistItem item) {
        fxLib.play_playlist_item(instance, item);
    }
}
