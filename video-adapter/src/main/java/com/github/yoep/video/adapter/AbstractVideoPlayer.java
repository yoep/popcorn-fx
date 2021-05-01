package com.github.yoep.video.adapter;

import com.github.yoep.video.adapter.listeners.VideoListener;
import com.github.yoep.video.adapter.state.VideoState;

import java.util.Collection;
import java.util.concurrent.ConcurrentLinkedQueue;

public abstract class AbstractVideoPlayer implements VideoPlayer {
    protected final Collection<VideoListener> listeners = new ConcurrentLinkedQueue<>();

    private VideoState videoState = VideoState.UNKNOWN;
    private Long time;
    private Long duration;

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
        listeners.forEach(e -> e.onStateChanged(playerState));
    }

    protected void setTime(Long time) {
        this.time = time;
        listeners.forEach(e -> e.onTimeChanged(time));
    }

    protected void setDuration(Long duration) {
        this.duration = duration;
        listeners.forEach(e -> e.onDurationChanged(duration));
    }

    //endregion
}
