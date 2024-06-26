package com.github.yoep.popcorn.backend.adapters.video;

import com.github.yoep.popcorn.backend.adapters.video.listeners.VideoListener;
import com.github.yoep.popcorn.backend.adapters.video.state.VideoState;
import javafx.scene.Node;

import java.io.File;

/**
 * The video playback is an embedded video backend which supports playback in the application itself.
 */
public interface VideoPlayback {
    //region Properties

    /**
     * Get the current state of the video playback.
     *
     * @return Returns the current state of the video playback.
     */
    VideoState getVideoState();

    /**
     * Get the current playback time of the video playback.
     *
     * @return Returns the milliseconds of the media playback.
     */
    long getTime();

    /**
     * Get the length of the current video playback.
     *
     * @return Returns the length in milliseonds of the video playback.
     */
    long getDuration();

    //endregion

    //region Getters & Setters

    /**
     * Get the unique name of the video player.
     *
     * @return Return the name of the video player.
     */
    
    String getName();

    /**
     * Get the summary of the video player.
     *
     * @return Returns the description of the video player.
     */
    String getDescription();

    /**
     * Check if the video player supports the given url.
     *
     * @param url The url to check the player compatibility of.
     * @return Returns true if the player supports the given url, else false.
     */
    boolean supports(String url);

    /**
     * Check if the video player has been initialized.
     *
     * @return Returns true if the video player has been initialized, else false.
     */
    boolean isInitialized();

    /**
     * Get the last error that occurred in the video player.
     *
     * @return Returns the last error of the video player (can be null).
     */
    Throwable getError();

    /**
     * Get the video surface of the video player.
     *
     * @return Returns the video surface of the video player.
     */
    Node getVideoSurface();

    //endregion

    //region Methods

    /**
     * Dispose the video player instance.
     */
    void dispose();

    /**
     * Add the given listener to the video player.
     *
     * @param listener The listener to register.
     */
    void addListener(VideoListener listener);

    /**
     * Remove the given listener from the video player.
     *
     * @param listener The listener to unregister.
     */
    void removeListener(VideoListener listener);

    /**
     * Play the given media url in the video player.
     * This will interrupt any media that is currently being played.
     *
     * @param url The media url to play.
     * @throws VideoPlayerNotInitializedException Is thrown when the video player has not yet been initialized.
     */
    void play(String url) throws VideoPlayerNotInitializedException;

    /**
     * Pause the media playback of the video player.
     *
     * @throws VideoPlayerNotInitializedException Is thrown when the video player has not yet been initialized.
     */
    void pause() throws VideoPlayerNotInitializedException;

    /**
     * Resume the media playback.
     *
     * @throws VideoPlayerNotInitializedException Is thrown when the video player has not yet been initialized.
     */
    void resume() throws VideoPlayerNotInitializedException;

    /**
     * Seek the given time in the current media playback.
     *
     * @param time The time to seek for in the current playback.
     * @throws VideoPlayerNotInitializedException Is thrown when the video player has not yet been initialized.
     */
    void seek(long time) throws VideoPlayerNotInitializedException;

    /**
     * Stop the current media playback in the video player.
     */
    void stop();

    /**
     * The new volume of the player.
     * The level vale must be between 0 and 100.
     *
     * @param volume The volume level of the player (0 = mute, 100 = max).
     */
    void volume(int volume);

    /**
     * Retrieve the volume level of the player.
     *
     * @return The volume level of the player (0 = mute, 100 = max).
     */
    int getVolume();

    /**
     * Check if the video player supports displaying subtitle files (SRT files).
     * This means that the {@link #subtitleFile(File)} and {@link #subtitleDelay(long)} can be used.
     * Otherwise, both methods might throw {@link UnsupportedOperationException} if this methods returns {@code false}.
     *
     * @return Returns true if the video player supports subtitle files, else false.
     */
    boolean supportsNativeSubtitleFile();

    /**
     * The subtitle file for the current playback.
     * Before using this method, check that native subtitles are supported with {@link #supportsNativeSubtitleFile()}.
     *
     * @param file The SRT subtitle file.
     * @throws UnsupportedOperationException Is thrown when native subtitle files are not supported by the player.
     */
    void subtitleFile(File file);

    /**
     * The subtitle delay in milliseconds.
     * Before using this method, check that native subtitles are supported with {@link #supportsNativeSubtitleFile()}.
     *
     * @param delay The delay of the subtitle.µ
     * @throws UnsupportedOperationException Is thrown when native subtitle files are not supported by the player.
     */
    void subtitleDelay(long delay);

    //endregion
}
