package com.github.yoep.popcorn.media.video.time;

import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;
import uk.co.caprica.vlcj.player.base.MediaPlayer;
import uk.co.caprica.vlcj.player.base.MediaPlayerEventAdapter;
import uk.co.caprica.vlcj.player.embedded.EmbeddedMediaPlayer;

import java.util.ArrayList;
import java.util.List;

@Slf4j
public class TimeHolder {
    private final List<TimeListener> listeners = new ArrayList<>();
    private long length;

    public TimeHolder(EmbeddedMediaPlayer mediaPlayer) {
        init(mediaPlayer);
    }

    public void addListener(TimeListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.add(listener);
        }
    }

    public long getLength() {
        return length;
    }

    private void init(EmbeddedMediaPlayer mediaPlayer) {
        mediaPlayer.events().addMediaPlayerEventListener(new MediaPlayerEventAdapter() {
            @Override
            public void timeChanged(MediaPlayer mediaPlayer, long newTime) {
                listeners.forEach(e -> invokeTimeChanged(e, newTime));
            }

            @Override
            public void lengthChanged(MediaPlayer mediaPlayer, long newLength) {
                length = newLength;
                listeners.forEach(e -> invokeLengthChanged(e, newLength));
            }
        });
    }

    private void invokeTimeChanged(TimeListener listener, long newTime) {
        try {
            listener.onTimeChanged(newTime);
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    private void invokeLengthChanged(TimeListener listener, long newLength) {
        try {
            listener.onLengthChanged(newLength);
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
    }
}
