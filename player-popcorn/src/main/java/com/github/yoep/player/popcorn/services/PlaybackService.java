package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.adapter.Player;
import com.github.yoep.player.adapter.PlayerManagerService;
import com.github.yoep.player.adapter.state.PlayerState;
import com.github.yoep.player.popcorn.PopcornPlayer;
import com.github.yoep.player.popcorn.PopcornPlayerNotFoundException;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

/**
 * The playback service is responsible for handling the playback actions of the player.
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class PlaybackService {
    private final PlayerManagerService playerService;

    //region Methods

    /**
     * Toggle the play/pause state of the {@link com.github.yoep.player.adapter.Player}.
     * When the player is {@link PlayerState#PAUSED}, it will resume the playback.
     * Otherwise, the player will be paused.
     */
    public void togglePlayerPlaybackState() {
        var player = getPlayer();

        if (player.getState() == PlayerState.PAUSED) {
            player.resume();
        } else {
            player.pause();
        }
    }

    /**
     * Resume the player playback.
     */
    public void resume() {
        getPlayer().resume();
    }

    /**
     * Pause the player playback.
     */
    public void pause() {
        getPlayer().pause();
    }

    /**
     * Stop the current video playback in the {@link Player}.
     */
    public void stop() {
        getPlayer().stop();
    }

    /**
     * Seek the given time in the playback.
     *
     * @param time The time to seek for.
     */
    public void seek(long time) {
        getPlayer().seek(time);
    }

    //endregion

    //region Functions

    private Player getPlayer() {
        return playerService.getById(PopcornPlayer.PLAYER_ID)
                .orElseThrow(PopcornPlayerNotFoundException::new);
    }

    //endregion
}
