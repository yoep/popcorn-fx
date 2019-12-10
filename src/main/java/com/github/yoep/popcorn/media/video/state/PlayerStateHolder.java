package com.github.yoep.popcorn.media.video.state;

import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;
import uk.co.caprica.vlcj.player.base.MediaPlayer;
import uk.co.caprica.vlcj.player.base.MediaPlayerEventAdapter;
import uk.co.caprica.vlcj.player.embedded.EmbeddedMediaPlayer;

import java.util.ArrayList;
import java.util.List;

@Slf4j
public class PlayerStateHolder {
    private final List<PlayerStateListener> listeners = new ArrayList<>();
    private PlayerState state;

    public PlayerStateHolder(EmbeddedMediaPlayer mediaPlayer) {
        init(mediaPlayer);
    }

    /**
     * Get the {@link PlayerState} from this holder.
     *
     * @return Returns the player state.
     */
    public PlayerState getState() {
        return state;
    }

    /**
     * Add the given listener to this video player listeners.
     *
     * @param listener The listener ta-hat needs to be registered.
     */
    public void addListener(PlayerStateListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.add(listener);
        }
    }

    private void init(EmbeddedMediaPlayer mediaPlayer) {
        mediaPlayer.events().addMediaPlayerEventListener(new MediaPlayerEventAdapter() {
            @Override
            public void playing(MediaPlayer mediaPlayer) {
                changeState(PlayerState.PLAYING);
            }

            @Override
            public void paused(MediaPlayer mediaPlayer) {
                changeState(PlayerState.PAUSED);
            }

            @Override
            public void stopped(MediaPlayer mediaPlayer) {
                changeState(PlayerState.STOPPED);
            }

            @Override
            public void finished(MediaPlayer mediaPlayer) {
                changeState(PlayerState.FINISHED);
            }

            @Override
            public void error(MediaPlayer mediaPlayer) {
                log.warn("Media player went into error state");
                changeState(PlayerState.ERROR);
            }
        });
    }

    private void changeState(PlayerState newState) {
        if (newState == state)
            return;

        PlayerState oldState = state;
        state = newState;

        synchronized (listeners) {
            listeners.forEach(e -> invokeListener(newState, oldState, e));
        }
    }

    private static void invokeListener(PlayerState newState, PlayerState oldState, PlayerStateListener listener) {
        try {
            listener.onChange(oldState, newState);
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
    }
}
