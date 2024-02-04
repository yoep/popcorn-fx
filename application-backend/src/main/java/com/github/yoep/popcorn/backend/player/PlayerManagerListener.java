package com.github.yoep.popcorn.backend.player;

public interface PlayerManagerListener {
    void activePlayerChanged(PlayerChanged playerChange);

    void playersChanged();
}
