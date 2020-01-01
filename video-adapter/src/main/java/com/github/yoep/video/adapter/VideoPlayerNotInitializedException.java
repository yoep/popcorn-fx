package com.github.yoep.video.adapter;

/**
 * Exception indicating that the video player instance has not yet been initialized.
 */
public class VideoPlayerNotInitializedException extends VideoPlayerException {
    private final VideoPlayer instance;

    public VideoPlayerNotInitializedException(VideoPlayer instance) {
        super("Video player has not been initialized");
        this.instance = instance;
    }

    /**
     * Get the video player instance that threw the exception.
     *
     * @return Returns the video player instance.
     */
    public VideoPlayer getInstance() {
        return instance;
    }
}
