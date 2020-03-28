package com.github.yoep.popcorn.view.services;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.resume.AutoResumeService;
import com.github.yoep.popcorn.messages.VideoMessage;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.subtitles.Subtitle;
import com.github.yoep.popcorn.subtitles.SubtitleService;
import com.github.yoep.popcorn.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.subtitles.models.SubtitleMatcher;
import com.github.yoep.popcorn.torrent.TorrentService;
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
import org.springframework.lang.Nullable;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;
import java.util.List;
import java.util.Optional;

@Slf4j
@Service
@RequiredArgsConstructor
public class VideoPlayerService {
    public static final String VIDEO_PLAYER_PROPERTY = "videoPlayer";
    public static final String SUBTITLE_PROPERTY = "subtitle";
    public static final String SUBTITLE_SIZE_PROPERTY = "subtitleSize";

    private final ActivityManager activityManager;
    private final AutoResumeService autoResumeService;
    private final FullscreenService fullscreenService;
    private final TorrentService torrentService;
    private final SettingsService settingsService;
    private final SubtitleService subtitleService;
    private final LocaleText localeText;
    private final List<VideoPlayer> videoPlayers;

    private final ObjectProperty<VideoPlayer> videoPlayer = new SimpleObjectProperty<>(this, VIDEO_PLAYER_PROPERTY);
    private final ObjectProperty<Subtitle> subtitle = new SimpleObjectProperty<>(this, SUBTITLE_PROPERTY, Subtitle.none());
    private final IntegerProperty subtitleSize = new SimpleIntegerProperty(this, SUBTITLE_SIZE_PROPERTY);
    private final ChangeListener<PlayerState> playerStateListener = (observable, oldValue, newValue) -> onPlayerStateChanged(newValue);
    private final ChangeListener<Number> timeListener = (observable, oldValue, newValue) -> time = newValue.longValue();
    private final ChangeListener<Number> durationListener = (observable, oldValue, newValue) -> duration = newValue.longValue();

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
        long newTime = getVideoPlayer().getTime() + millis;
        long duration = getVideoPlayer().getDuration();

        if (newTime > duration)
            newTime = duration;
        if (newTime < 0)
            newTime = 0;

        getVideoPlayer().seek(newTime);
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
     * Close the video player.
     */
    public void close() {
        onClose();
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        log.trace("Initializing video player service");
        initializeListeners();
        initializeSubtitleSize();
        initializeVideoListeners();
    }

    private void initializeListeners() {
        activityManager.register(PlayVideoActivity.class, this::onPlayVideo);
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
    }

    private void onPlayVideo(PlayVideoActivity activity) {
        this.videoChangeTime = System.currentTimeMillis();

        // check if the activity contains media information
        // if so, play the video as a media instead of a plain url playback
        if (activity instanceof PlayMediaActivity) {
            var mediaActivity = (PlayMediaActivity) activity;
            onPlayMedia(mediaActivity);
            return;
        }

        log.debug("Received play video activity for url \"{}\" and title \"{}\"", activity.getUrl(), activity.getTitle());
        playUrl(activity.getUrl());
    }

    private void onPlayMedia(PlayMediaActivity activity) {
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

    private void onSubtitleChanged(SubtitleInfo subtitle) {
        log.debug("Downloading subtitle \"{}\" for video playback", subtitle);
        var matcher = SubtitleMatcher.from(FilenameUtils.getBaseName(url), quality);

        subtitleService.downloadAndParse(subtitle, matcher).whenComplete((subtitles, throwable) -> {
            if (throwable != null) {
                log.error("Video subtitle failed, " + throwable.getMessage(), throwable);
                activityManager.register((ErrorNotificationActivity) () -> localeText.get(VideoMessage.SUBTITLE_DOWNLOAD_FILED));
            } else {

            }
        });
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

        torrentService.stopStream();
        activityManager.register(new ClosePlayerActivity() {
            @Override
            public String getUrl() {
                return url;
            }

            @Override
            public Optional<Media> getMedia() {
                return Optional.ofNullable(media);
            }

            @Override
            public Optional<String> getQuality() {
                return Optional.ofNullable(quality);
            }

            @Override
            public long getTime() {
                return Optional.ofNullable(time)
                        .orElse(UNKNOWN);
            }

            @Override
            public long getDuration() {
                return Optional.ofNullable(duration)
                        .orElse(UNKNOWN);
            }
        });

        reset();
    }

    private void playUrl(String url) {
        updateActiveVideoPlayer(url);
        this.url = url;
        getVideoPlayer().play(url);

        var filename = FilenameUtils.getName(url);

        // check if we need to auto resume the current video playback
        Platform.runLater(() -> {
            if (media != null) {
                autoResumeService.getResumeTimestamp(media.getId(), filename)
                        .ifPresent(getVideoPlayer()::seek);
            } else {
                autoResumeService.getResumeTimestamp(filename)
                        .ifPresent(getVideoPlayer()::seek);
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

    private void onVideoStopped() {
        // check if the video has been started for more than 30 sec before exiting the video player
        // this should fix the issue of the video player closing directly in some cases
        if (System.currentTimeMillis() - videoChangeTime <= 30000)
            return;

        close();
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
