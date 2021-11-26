package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.adapter.Player;
import com.github.yoep.player.adapter.state.PlayerState;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
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
    private final PopcornPlayer player;

    //region Methods

    /**
     * Toggle the play/pause state of the {@link com.github.yoep.player.adapter.Player}.
     * When the player is {@link PlayerState#PAUSED}, it will resume the playback.
     * Otherwise, the player will be paused.
     */
    public void togglePlayerPlaybackState() {
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
        player.resume();
    }

    /**
     * Pause the player playback.
     */
    public void pause() {
        player.pause();
    }

    /**
     * Stop the current video playback in the {@link Player}.
     */
    public void stop() {
        player.stop();
    }

    /**
     * Seek the given time in the playback.
     *
     * @param time The time to seek for.
     */
    public void seek(long time) {
        player.seek(time);
    }

    //endregion
}
