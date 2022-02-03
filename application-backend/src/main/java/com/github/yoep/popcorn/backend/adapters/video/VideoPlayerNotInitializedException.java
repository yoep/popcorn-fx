package com.github.yoep.popcorn.backend.adapters.video;

/**
 * Exception indicating that the video player instance has not yet been initialized.
 */
public class VideoPlayerNotInitializedException extends VideoPlayerException {
    private final VideoPlayback instance;

    public VideoPlayerNotInitializedException(VideoPlayback instance) {
        super("Video player has not been initialized");
        this.instance = instance;
    }

    /**
     * Get the video player instance that threw the exception.
     *
     * @return Returns the video player instance.
     */
    public VideoPlayback getInstance() {
        return instance;
    }
}
