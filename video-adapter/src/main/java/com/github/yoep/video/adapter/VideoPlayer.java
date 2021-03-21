package com.github.yoep.video.adapter;

import com.github.yoep.video.adapter.state.PlayerState;
import javafx.beans.property.ReadOnlyLongProperty;
import javafx.beans.property.ReadOnlyObjectProperty;
import javafx.scene.Node;

import java.io.File;

/**
 * The video player is a embedded video backend which supports playback in the application itself.
 */
public interface VideoPlayer {
    String PLAYER_STATE_PROPERTY = "playerState";
    String TIME_PROPERTY = "time";
    String DURATION_PROPERTY = "duration";

    //region Properties

    /**
     * Get the current state of the video player.
     *
     * @return Returns the current state of the video player.
     */
    PlayerState getPlayerState();

    /**
     * Get the video player state property.
     *
     * @return Returns the read-only player state property.
     */
    ReadOnlyObjectProperty<PlayerState> playerStateProperty();

    /**
     * Get the current playback time of the video player.
     *
     * @return Returns the milliseconds of the media playback.
     */
    long getTime();

    /**
     * Get the time property of the video player.
     *
     * @return Returns the read-only time property.
     */
    ReadOnlyLongProperty timeProperty();

    /**
     * Get the length of the current media playback.
     *
     * @return Returns the length in milliseonds of the media playback.
     */
    long getDuration();

    /**
     * Get the length property of the video player.
     *
     * @return Returns the read-only length property.
     */
    ReadOnlyLongProperty durationProperty();

    //endregion

    //region Getters & Setters

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
