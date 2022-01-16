package com.github.yoep.popcorn.ui.view.listeners;

import com.github.yoep.popcorn.ui.view.model.AboutDetail;

import java.util.List;

public interface AboutSectionListener {
    /**
     * Invoked when the players list has been changed.
     *
     * @param players The new available players.
     */
    void onPlayersChanged(List<AboutDetail> players);
}
