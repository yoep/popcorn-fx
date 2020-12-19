package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.ui.events.ClosePlayerEvent;
import com.github.yoep.popcorn.ui.events.PlayMediaEvent;
import com.github.yoep.popcorn.ui.events.PlayVideoEvent;
import com.github.yoep.popcorn.ui.events.PlayerStoppedEvent;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.media.resume.AutoResumeService;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.view.listeners.VideoPlayerListener;
import com.github.yoep.torrent.adapter.TorrentStreamService;
import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.VideoPlayerNotInitializedException;
import com.github.yoep.video.adapter.state.PlayerState;
import javafx.application.Platform;
import javafx.beans.property.ReadOnlyBooleanProperty;
import javafx.beans.value.ChangeListener;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.event.EventListener;
import org.springframework.lang.Nullable;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.function.Consumer;

@Slf4j
@Service
@RequiredArgsConstructor
public class VideoPlayerService {

    private final ApplicationEventPublisher eventPublisher;
    private final AutoResumeService autoResumeService;
    private final FullscreenService fullscreenService;
    private final TorrentStreamService torrentStreamService;
    private final SettingsService settingsService;
    private final VideoPlayerManagerService videoPlayerManagerService;
    private final VideoPlayerSubtitleService videoPlayerSubtitleService;

    private final ChangeListener<PlayerState> playerStateListener = (observable, oldValue, newValue) -> onPlayerStateChanged(newValue);
    private final ChangeListener<Number> timeListener = (observable, oldValue, newValue) -> onTimeChanged(newValue);
    private final ChangeListener<Number> durationListener = (observable, oldValue, newValue) -> onDurationChanged(newValue);
    private final List<VideoPlayerListener> listeners = new ArrayList<>();

    @Nullable
    private Media media;
    private String quality;
    private String url;
    private Long time;
    private Long duration;

    //region Properties

    /**
     * Get the fullscreen property of the application.
     *
     * @return Returns the fullscreen property.
     */
    public ReadOnlyBooleanProperty fullscreenProperty() {
        return fullscreenService.fullscreenProperty();
    }

    //endregion

    //region Methods

    /**
     * Register the given listener to this video player service.
     *
     * @param listener The listener to add.
     */
    public void addListener(VideoPlayerListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.add(listener);
        }
    }

    /**
     * Resume the media playback.
     *
     * @throws VideoPlayerNotInitializedException Is thrown when the video player has not yet been initialized.
     */
    public void resume() {
        videoPlayerManagerService.getActivePlayer()
                .ifPresent(VideoPlayer::resume);
    }

    /**
     * Pause the media playback of the video player.
     *
     * @throws VideoPlayerNotInitializedException Is thrown when the video player has not yet been initialized.
     */
    public void pause() {
        videoPlayerManagerService.getActivePlayer()
                .ifPresent(VideoPlayer::pause);
    }

    /**
     * Seek the given time in the current media playback.
     *
     * @param time The time to seek for in the current playback.
     * @throws VideoPlayerNotInitializedException Is thrown when the video player has not yet been initialized.
     */
    public void seek(long time) {
        videoPlayerManagerService.getActivePlayer()
                .ifPresent(e -> e.seek(time));
    }

    /**
     * Stop the video playback.
     */
    public void stop() {
        onStop();
    }

    /**
     * Get the last error that occurred in the video player.
     *
     * @return Returns the last error of the video player (can be null).
     */
    public Throwable getError() {
        return videoPlayerManagerService.getActivePlayer()
                .map(VideoPlayer::getError)
                .orElse(null);
    }

    /**
     * Increase or decrease the video time by the given millis offset.
     *
     * @param millis The millis offset in regards to the current video time.
     */
    public void videoTimeOffset(long millis) {
        log.trace("Updating video time with {} offset", millis);
        var videoPlayer = videoPlayerManagerService.getVideoPlayer();

        // check if a video player is currently active
        // if not, ignore the time offset update
        if (videoPlayer == null) {
            log.warn("Unable to update video time offset, video player is unknown (null)");
            return;
        }

        var newTime = videoPlayer.getTime() + millis;
        var duration = videoPlayer.getDuration();

        if (newTime > duration)
            newTime = duration;
        if (newTime < 0)
            newTime = 0;

        videoPlayer.seek(newTime);
    }

    /**
     * Change the current play/pause state of the video player.
     */
    public void changePlayPauseState() {
        var videoPlayer = videoPlayerManagerService.getVideoPlayer();

        // check if the video player present
        // if not, ignore this action
        if (videoPlayer == null)
            return;

        if (videoPlayer.getPlayerState() == PlayerState.PAUSED) {
            log.trace("Video player state is being changed to \"resume\"");
            videoPlayer.resume();
        } else {
            log.trace("Video player state is being changed to \"paused\"");
            videoPlayer.pause();
        }
    }

    /**
     * Toggle the fullscreen mode of the application.
     */
    public void toggleFullscreen() {
        fullscreenService.toggle();
    }

    /**
     * Stop the video playback and close the video player.
     */
    public void close() {
        onClose();
    }

    @EventListener
    public void onPlayVideo(PlayVideoEvent activity) {
        // check if the activity contains media information
        // if so, play the video as a media instead of a plain url playback
        if (activity instanceof PlayMediaEvent) {
            var mediaActivity = (PlayMediaEvent) activity;
            onPlayMedia(mediaActivity);
            return;
        }

        log.debug("Received play video activity for url \"{}\" and title \"{}\"", activity.getUrl(), activity.getTitle());
        playUrl(activity.getUrl());
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        log.trace("Initializing video player service");
        initializeVideoListeners();
    }

    private void initializeVideoListeners() {
        videoPlayerManagerService.videoPlayerProperty().addListener((observable, oldValue, newValue) -> {
            if (oldValue != null) {
                oldValue.playerStateProperty().removeListener(playerStateListener);
                oldValue.timeProperty().removeListener(timeListener);
                oldValue.durationProperty().removeListener(durationListener);
            }

            newValue.playerStateProperty().addListener(playerStateListener);
            newValue.timeProperty().addListener(timeListener);
            newValue.durationProperty().addListener(durationListener);
        });
    }

    //endregion

    //region Functions

    private void onPlayerStateChanged(PlayerState newValue) {
        if (newValue == PlayerState.STOPPED) {
            onVideoStopped();
        }

        invokeListeners(e -> e.onPlayerStateChanged(newValue));
    }

    private void onPlayMedia(PlayMediaEvent activity) {
        log.debug("Received play media activity for url {}, quality {} and media {}", activity.getUrl(), activity.getQuality(),
                activity.getMedia());
        this.media = activity.getMedia();
        this.quality = activity.getQuality();
        var activitySubtitle = activity.getSubtitle();

        // check if a subtitle was selected
        if (activitySubtitle.isPresent() && !activitySubtitle.get().isNone()) {
            videoPlayerSubtitleService.setSubtitle(activitySubtitle.get());
        }

        playUrl(activity.getUrl());
    }

    private void onStop() {
        // cache the current time & duration
        // the stop on the video player might reset the time & duration to 0
        var time = this.time;
        var duration = this.duration;

        // stop the video player in case it might be still playing
        Optional.ofNullable(videoPlayerManagerService.getVideoPlayer())
                .ifPresent(VideoPlayer::stop);

        log.trace("Publishing player stopped event with info: [time: {}, duration: {}]", time, duration);
        eventPublisher.publishEvent(PlayerStoppedEvent.builder()
                .source(this)
                .url(url)
                .media(media)
                .quality(quality)
                .time(Optional.ofNullable(time).orElse(PlayerStoppedEvent.UNKNOWN))
                .duration(Optional.ofNullable(duration).orElse(PlayerStoppedEvent.UNKNOWN))
                .build());
        torrentStreamService.stopAllStreams();
    }

    private void onClose() {
        // stop the playback
        onStop();

        // close the player
        eventPublisher.publishEvent(new ClosePlayerEvent(this));

        reset();
    }

    private void playUrl(String url) {
        videoPlayerManagerService.updateActiveVideoPlayer(url);
        this.url = url;
        var videoPlayer = videoPlayerManagerService.getVideoPlayer();
        var filename = FilenameUtils.getName(url);
        var playbackSettings = settingsService.getSettings().getPlaybackSettings();

        // check if a video player was found
        // if not, stop the play url execution
        if (videoPlayer == null) {
            log.error("Failed to play url, no video player found");
            return;
        }

        // start the playback of the video url
        videoPlayer.play(url);

        // enter fullscreen mode on the start of the playback if configured
        if (playbackSettings.isFullscreen())
            fullscreenService.fullscreen(true);

        // check if we need to auto resume the current video playback
        Platform.runLater(() -> {
            if (media != null) {
                autoResumeService.getResumeTimestamp(media.getId(), filename)
                        .ifPresent(videoPlayer::seek);
            } else {
                autoResumeService.getResumeTimestamp(filename)
                        .ifPresent(videoPlayer::seek);
            }
        });
    }

    private void onTimeChanged(Number newValue) {
        time = newValue.longValue();

        invokeListeners(e -> e.onTimeChanged(newValue));
    }

    private void onDurationChanged(Number newValue) {
        duration = newValue.longValue();

        invokeListeners(e -> e.onDurationChanged(newValue));
    }

    private void onVideoStopped() {
        var isDurationUnknown = videoPlayerManagerService.getActivePlayer()
                .map(e -> e.getDuration() == 0)
                .orElse(false);

        // check if the duration is not 0 for the active player
        // if so, don't close the player and wait
        // the playback of youtube videos in VLC will report a STOPPED event before actually starting the video playback
        // this causes the player to instantly close before the actual video playback has started
        if (isDurationUnknown)
            return;

        close();
    }

    private void invokeListeners(Consumer<VideoPlayerListener> action) {
        synchronized (listeners) {
            for (VideoPlayerListener listener : listeners) {
                try {
                    action.accept(listener);
                } catch (Exception ex) {
                    log.error("Error occurred while invoking listener, " + ex.getMessage(), ex);
                }
            }
        }
    }

    private void reset() {
        this.url = null;
        this.media = null;
        this.quality = null;
        this.time = null;
        this.duration = null;
    }

    //endregion
}
