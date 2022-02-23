package com.github.yoep.video.vlc;

import com.github.yoep.popcorn.backend.adapters.video.AbstractVideoPlayer;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayerNotInitializedException;
import com.github.yoep.popcorn.backend.adapters.video.listeners.VideoListener;
import com.github.yoep.popcorn.backend.adapters.video.state.VideoState;
import javafx.scene.Node;

import java.io.File;

public class VideoPlayerVlcError extends AbstractVideoPlayer implements VideoPlayback {
    public VideoPlayerVlcError() {
        setVideoState(VideoState.ERROR);
    }

    @Override
    public String getName() {
        return VideoPlayerVlc.NAME;
    }

    @Override
    public String getDescription() {
        return VideoPlayerVlc.DESCRIPTION;
    }

    @Override
    public boolean supports(String url) {
        return false;
    }

    @Override
    public boolean isInitialized() {
        return false;
    }

    @Override
    public Throwable getError() {
        return null;
    }

    @Override
    public Node getVideoSurface() {
        return null;
    }

    @Override
    public void dispose() {

    }

    @Override
    public void addListener(VideoListener listener) {

    }

    @Override
    public void removeListener(VideoListener listener) {

    }

    @Override
    public void play(String url) throws VideoPlayerNotInitializedException {

    }

    @Override
    public void pause() throws VideoPlayerNotInitializedException {

    }

    @Override
    public void resume() throws VideoPlayerNotInitializedException {

    }

    @Override
    public void seek(long time) throws VideoPlayerNotInitializedException {

    }

    @Override
    public void stop() {

    }

    @Override
    public boolean supportsNativeSubtitleFile() {
        return false;
    }

    @Override
    public void subtitleFile(File file) {

    }

    @Override
    public void subtitleDelay(long delay) {

    }
}
