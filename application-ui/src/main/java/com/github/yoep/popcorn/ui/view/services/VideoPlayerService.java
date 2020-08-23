package com.github.yoep.popcorn.ui.view.services;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.events.ClosePlayerEvent;
import com.github.yoep.popcorn.ui.events.ErrorNotificationEvent;
import com.github.yoep.popcorn.ui.events.PlayMediaEvent;
import com.github.yoep.popcorn.ui.events.PlayVideoEvent;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.media.resume.AutoResumeService;
import com.github.yoep.popcorn.ui.messages.VideoMessage;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.subtitles.Subtitle;
import com.github.yoep.popcorn.ui.subtitles.SubtitlePickerService;
import com.github.yoep.popcorn.ui.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleMatcher;
import com.github.yoep.popcorn.ui.view.listeners.VideoPlayerListener;
import com.github.yoep.torrent.adapter.TorrentStreamService;
import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.VideoPlayerException;
import com.github.yoep.video.adapter.VideoPlayerNotInitializedException;
import com.github.yoep.video.adapter.state.PlayerState;
import javafx.application.Platform;
import javafx.beans.property.*;
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
import javax.annotation.PreDestroy;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.function.Consumer;

@Slf4j
@Service
@RequiredArgsConstructor
public class VideoPlayerService {
    public static final String VIDEO_PLAYER_PROPERTY = "videoPlayer";
    public static final String SUBTITLE_PROPERTY = "subtitle";
    public static final String SUBTITLE_SIZE_PROPERTY = "subtitleSize";

    private final ApplicationEventPublisher eventPublisher;
    private final AutoResumeService autoResumeService;
    private final FullscreenService fullscreenService;
    private final TorrentStreamService torrentStreamService;
    private final SettingsService settingsService;
    private final SubtitleService subtitleService;
    private final SubtitlePickerService subtitlePickerService;
    private final LocaleText localeText;
    private final List<VideoPlayer> videoPlayers;

    private final ObjectProperty<VideoPlayer> videoPlayer = new SimpleObjectProperty<>(this, VIDEO_PLAYER_PROPERTY);
    private final ObjectProperty<Subtitle> subtitle = new SimpleObjectProperty<>(this, SUBTITLE_PROPERTY, Subtitle.none());
    private final IntegerProperty subtitleSize = new SimpleIntegerProperty(this, SUBTITLE_SIZE_PROPERTY);
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
    private long videoChangeTime;

    //region Properties

    /**
     * Get the current active video player of the service.
     *
     * @return Returns the active video player.
     */
    @Nullable
    public VideoPlayer getVideoPlayer() {
        return videoPlayer.get();
    }

    /**
     * Get the video player property.
     *
     * @return Returns the video player property.
     */
    public ReadOnlyObjectProperty<VideoPlayer> videoPlayerProperty() {
        return videoPlayer;
    }

    /**
     * Get the current subtitle of the video player.
     *
     * @return Returns the subtitle.
     */
    public Subtitle getSubtitle() {
        return subtitle.get();
    }

    /**
     * Get the subtitle property.
     *
     * @return Returns the subtitle property.
     */
    public ObjectProperty<Subtitle> subtitleProperty() {
        return subtitle;
    }

    /**
     * Set the subtitle for the video player.
     *
     * @param subtitle The subtitle for the video player.
     */
    public void setSubtitle(Subtitle subtitle) {
        Assert.notNull(subtitle, "subtitle cannot be null");
        this.subtitle.set(subtitle);
    }

    /**
     * Get the subtitle font size.
     *
     * @return Returns the subtitle font size.
     */
    public int getSubtitleSize() {
        return subtitleSize.get();
    }

    /**
     * Get the subtitle size property.
     *
     * @return Returns the subtitle size property.
     */
    public IntegerProperty subtitleSizeProperty() {
        return subtitleSize;
    }

    /**
     * Set the new subtitle size.
     *
     * @param subtitleSize The new subtitle size.
     */
    public void setSubtitleSize(int subtitleSize) {
        this.subtitleSize.set(subtitleSize);
    }

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
        var videoPlayer = getVideoPlayer();

        if (videoPlayer != null)
            videoPlayer.resume();
    }

    /**
     * Pause the media playback of the video player.
     *
     * @throws VideoPlayerNotInitializedException Is thrown when the video player has not yet been initialized.
     */
    public void pause() {
        var videoPlayer = getVideoPlayer();

        if (videoPlayer != null)
            videoPlayer.pause();
    }

    /**
     * Seek the given time in the current media playback.
     *
     * @param time The time to seek for in the current playback.
     * @throws VideoPlayerNotInitializedException Is thrown when the video player has not yet been initialized.
     */
    public void seek(long time) {
        var videoPlayer = getVideoPlayer();

        if (videoPlayer != null)
            videoPlayer.seek(time);
    }

    /**
     * Get the last error that occurred in the video player.
     *
     * @return Returns the last error of the video player (can be null).
     */
    public Throwable getError() {
        var videoPlayer = getVideoPlayer();

        if (videoPlayer != null)
            return videoPlayer.getError();

        return null;
    }

    /**
     * Increase or decrease the video time by the given millis offset.
     *
     * @param millis The millis offset in regards to the current video time.
     */
    public void videoTimeOffset(long millis) {
        log.trace("Updating video time with {} offset", millis);
        var videoPlayer = getVideoPlayer();

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
        var videoPlayer = getVideoPlayer();

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
     * Set the new subtitle info for the video playback.
     * This method will automatically download and parse the new subtitle.
     *
     * @param subtitleInfo The new subtitle info.
     */
    public void setSubtitle(SubtitleInfo subtitleInfo) {
        onSubtitleChanged(subtitleInfo);
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
        this.videoChangeTime = System.currentTimeMillis();

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
        initializeSubtitleSize();
        initializeVideoListeners();
    }

    private void initializeSubtitleSize() {
        subtitleSize.set(settingsService.getSettings().getSubtitleSettings().getFontSize());
    }

    private void initializeVideoListeners() {
        videoPlayerProperty().addListener((observable, oldValue, newValue) -> {
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

    //region PreDestroy

    @PreDestroy
    private void dispose() {
        videoPlayers.forEach(VideoPlayer::dispose);
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
            setSubtitle(activitySubtitle.get());
        }

        playUrl(activity.getUrl());
    }

    private void onSubtitleChanged(SubtitleInfo subtitleInfo) {
        // check if the subtitle is being disabled
        // if so, update the subtitle to none and ignore the subtitle download & parsing
        if (subtitleInfo == null || subtitleInfo.isNone()) {
            disableSubtitleTrack();
            return;
        }

        final var imdbId = subtitleInfo.getImdbId();
        final var language = subtitleInfo.getLanguage();

        // check if the subtitle is a custom subtitle and doesn't contain any files yet
        // if so, pause the playback and let the user pick a custom subtitle file
        // if the custom subtitle contains files, than the passed subtitle file is from the details components
        if (subtitleInfo.isCustom() && subtitleInfo.getFiles().isEmpty()) {
            // pause the video playback as a popup will be shown
            pause();

            // show the subtitle picker popup
            var customSubtitle = subtitlePickerService.pickCustomSubtitle();

            if (customSubtitle.isPresent()) {
                // overrule the given subtitleInfo with the custom subtitle file picked by the user
                subtitleInfo = customSubtitle.get();
            } else {
                disableSubtitleTrack();
                return;
            }

            // resume the video playback
            resume();
        }

        log.debug("Downloading subtitle \"{}\" for video playback", subtitleInfo);
        var matcher = SubtitleMatcher.from(FilenameUtils.getBaseName(url), quality);

        subtitleService.downloadAndParse(subtitleInfo, matcher).whenComplete((subtitles, throwable) -> {
            if (throwable == null) {
                log.debug("Subtitle (imdbId: {}, language: {}) has been downloaded with success", imdbId, language);
                this.setSubtitle(subtitles);
            } else {
                log.error("Video subtitle failed, " + throwable.getMessage(), throwable);
                eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(VideoMessage.SUBTITLE_DOWNLOAD_FILED)));
            }
        });
    }

    private void disableSubtitleTrack() {
        log.debug("Disabling the subtitle track for the video playback");
        setSubtitle(Subtitle.NONE);
    }

    private void onClose() {
        // keep a copy of the information for later use in the activity
        var url = this.url;
        var media = this.media;
        var quality = this.quality;
        var time = this.time;
        var duration = this.duration;

        // stop the video player in case it might be still playing
        Optional.ofNullable(getVideoPlayer())
                .ifPresent(VideoPlayer::stop);

        eventPublisher.publishEvent(ClosePlayerEvent.builder()
                .source(this)
                .url(url)
                .media(media)
                .quality(quality)
                .time(Optional.ofNullable(time).orElse(ClosePlayerEvent.UNKNOWN))
                .duration(Optional.ofNullable(duration).orElse(ClosePlayerEvent.UNKNOWN))
                .build());
        torrentStreamService.stopAllStreams();

        reset();
    }

    private void playUrl(String url) {
        updateActiveVideoPlayer(url);
        this.url = url;
        var videoPlayer = getVideoPlayer();
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

    private void updateActiveVideoPlayer(String url) {
        var videoPlayer = videoPlayers.stream()
                .filter(e -> e.supports(url))
                .findFirst()
                .orElseThrow(() -> new VideoPlayerException("No compatible video player found for " + url));

        // check if the video player is the same
        // if so, do not update the active video player
        if (videoPlayer == getVideoPlayer())
            return;

        this.videoPlayer.set(videoPlayer);
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
        // check if the video has been started for more than 30 sec before exiting the video player
        // this should fix the issue of the video player closing directly in some cases
        if (System.currentTimeMillis() - videoChangeTime <= 30000)
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
