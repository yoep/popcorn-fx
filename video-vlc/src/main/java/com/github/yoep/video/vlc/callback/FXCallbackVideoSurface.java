package com.github.yoep.video.vlc.callback;

import lombok.extern.slf4j.Slf4j;
import uk.co.caprica.vlcj.player.embedded.videosurface.CallbackVideoSurface;
import uk.co.caprica.vlcj.player.embedded.videosurface.VideoSurfaceAdapters;

@Slf4j
public class FXCallbackVideoSurface extends CallbackVideoSurface {
    private final FXRenderCallback renderCallback;

    public FXCallbackVideoSurface(FXRenderCallback renderCallback) {
        super(renderCallback.getBufferFormat(), renderCallback, true, VideoSurfaceAdapters.getVideoSurfaceAdapter());
        this.renderCallback = renderCallback;
    }

    /**
     * Render a new video frame on the video surface.
     */
    public void render() {
        renderCallback.renderFrame();
    }

    /**
     * Reset the video surface to a black frame.
     */
    public void reset() {
        renderCallback.renderBlackFrame();
    }
}

