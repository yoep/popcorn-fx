package com.github.yoep.popcorn.backend.adapters.video;

import com.github.yoep.popcorn.backend.adapters.video.listeners.VideoListener;
import com.github.yoep.popcorn.backend.adapters.video.state.VideoState;
import lombok.extern.slf4j.Slf4j;

import java.util.Collection;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.function.Consumer;

@Slf4j
public abstract class AbstractVideoPlayer implements VideoPlayback {
    protected final Collection<VideoListener> listeners = new ConcurrentLinkedQueue<>();

    VideoState videoState = VideoState.UNKNOWN;
    Long time;
    Long duration;

    //region Properties

    @Override
    public VideoState getVideoState() {
        return videoState;
    }

    @Override
    public long getTime() {
        return time;
    }

    @Override
    public long getDuration() {
        return duration;
    }

    protected void setVideoState(VideoState playerState) {
        this.videoState = playerState;
        invokeListeners(e -> e.onStateChanged(playerState));
    }

    protected void setTime(Long time) {
        this.time = time;
        invokeListeners(e -> e.onTimeChanged(time));
    }

    protected void setDuration(Long duration) {
        this.duration = duration;
        invokeListeners(e -> e.onDurationChanged(duration));
    }

    protected void invokeListeners(Consumer<VideoListener> action) {
        listeners.forEach(e -> {
            try {
                action.accept(e);
            } catch (Exception ex) {
                log.error(ex.getMessage(), ex);
            }
        });
    }

    //endregion
}
