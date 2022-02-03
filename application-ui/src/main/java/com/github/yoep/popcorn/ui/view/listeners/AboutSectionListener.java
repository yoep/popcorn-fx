package com.github.yoep.popcorn.ui.view.listeners;

import com.github.yoep.popcorn.backend.info.ComponentInfo;

import java.util.List;

public interface AboutSectionListener {
    /**
     * Invoked when the players list has been changed.
     *
     * @param players The new available players.
     */
    void onPlayersChanged(List<ComponentInfo> players);

    /**
     * Invoked when the video players have been changed.
     *
     * @param videoPlayers The new list of video players.
     */
    void onVideoPlayersChanged(List<ComponentInfo> videoPlayers);
}
