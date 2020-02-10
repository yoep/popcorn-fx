package com.github.yoep.popcorn.controllers.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.ClosePlayerActivity;
import com.github.yoep.popcorn.activities.PlayMediaActivity;
import com.github.yoep.popcorn.activities.PlayVideoActivity;
import com.github.yoep.popcorn.controllers.components.PlayerControlsComponent;
import com.github.yoep.popcorn.controllers.components.PlayerControlsListener;
import com.github.yoep.popcorn.controllers.components.PlayerHeaderComponent;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.resume.AutoResumeService;
import com.github.yoep.popcorn.messages.VideoMessage;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.subtitles.SubtitleService;
import com.github.yoep.popcorn.subtitles.controls.SubtitleTrack;
import com.github.yoep.popcorn.subtitles.models.DecorationType;
import com.github.yoep.popcorn.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.VideoPlayerException;
import com.github.yoep.video.adapter.state.PlayerState;
import javafx.animation.FadeTransition;
import javafx.animation.PauseTransition;
import javafx.application.Platform;
import javafx.beans.value.ChangeListener;
import javafx.event.Event;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Cursor;
import javafx.scene.Node;
import javafx.scene.canvas.Canvas;
import javafx.scene.control.Label;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.Pane;
import javafx.scene.media.MediaView;
import javafx.scene.text.FontWeight;
import javafx.scene.web.WebView;
import javafx.util.Duration;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;
import org.apache.commons.lang3.StringUtils;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;
import java.net.URL;
import java.util.List;
import java.util.Optional;
import java.util.ResourceBundle;

@Slf4j
@Component
@RequiredArgsConstructor
public class PlayerSectionController implements Initializable {
    private static final int OVERLAY_FADE_DURATION = 1500;
    private static final int INFO_FADE_DURATION = 2000;

    private final PauseTransition idleTimer = new PauseTransition(Duration.seconds(3));
    private final PauseTransition offsetTimer = new PauseTransition(Duration.seconds(2));

    private final ActivityManager activityManager;
    private final TaskExecutor taskExecutor;
    private final TorrentService torrentService;
    private final SubtitleService subtitleService;
    private final SettingsService settingsService;
    private final AutoResumeService autoResumeService;
    private final PlayerHeaderComponent playerHeader;
    private final PlayerControlsComponent playerControls;
    private final List<VideoPlayer> videoPlayers;
    private final LocaleText localeText;

    private VideoPlayer videoPlayer;
    private ChangeListener<PlayerState> playerStateListener;
    private ChangeListener<Number> timeListener;
    private ChangeListener<Number> durationListener;

    private String url;
    private Media media;
    private String quality;
    private long videoChangeTime;

    @FXML
    private Pane playerPane;
    @FXML
    private Pane playerHeaderPane;
    @FXML
    private Pane playerVideoOverlay;
    @FXML
    private Pane playerControlsPane;
    @FXML
    private Pane videoView;
    @FXML
    private Label subtitleOffset;
    @FXML
    private Label errorText;
    @FXML
    private Pane bufferIndicator;
    @FXML
    private SubtitleTrack subtitleTrack;

    //region Methods

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        log.trace("Initializing video player component for JavaFX");
        initializeSceneEvents();
        initializeSubtitleTrack();
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        log.trace("Initializing video player component for Spring");
        initializeListeners();
        initializeVideoListeners();
    }

    private void initializeListeners() {
        activityManager.register(PlayVideoActivity.class, this::onPlayVideo);
        playerHeader.addListener(this::close);
        playerControls.addListener(new PlayerControlsListener() {
            @Override
            public void onSubtitleChanged(SubtitleInfo subtitle) {
                PlayerSectionController.this.onSubtitleChanged(subtitle);
            }

            @Override
            public void onSubtitleSizeChanged(int pixelChange) {
                subtitleTrack.setFontSize(subtitleTrack.getFontSize() + pixelChange);
            }

            @Override
            public void onTimeChanged(long time) {
                videoPlayer.seek(time);
            }

            @Override
            public void onPlayPauseClicked() {
                changePlayPauseState();
            }
        });
    }

    private void initializeVideoListeners() {
        playerStateListener = (observable, oldValue, newValue) -> {
            if (newValue != PlayerState.ERROR)
                log.debug("Video player state changed to {}", newValue);

            bufferIndicator.setVisible(newValue == PlayerState.BUFFERING);

            switch (newValue) {
                case ERROR:
                    log.error("Video player state changed to {}", newValue);
                    Throwable error = videoPlayer.getError();

                    if (error != null)
                        log.error(error.getMessage(), error);

                    Platform.runLater(() -> errorText.setText(localeText.get(VideoMessage.VIDEO_ERROR)));
                    break;
                case STOPPED:
                    onVideoStopped();
                    break;
            }

            playerControls.onPlayerStateChanged(newValue);
        };
        timeListener = (observable, oldValue, newValue) -> {
            subtitleTrack.onTimeChanged(newValue.longValue());

            playerControls.onTimeChanged(newValue);
        };
        durationListener = (observableValue, oldValue, newValue) -> playerControls.onDurationChanged(newValue);
    }

    //endregion

    //region PreDestroy

    @PreDestroy
    private void dispose() {
        videoPlayers.forEach(VideoPlayer::dispose);
    }

    //endregion

    //region Functions

    private void initializeSceneEvents() {
        playerPane.setOnKeyReleased(event -> {
            switch (event.getCode()) {
                case LEFT:
                case KP_LEFT:
                    increaseVideoTime(-5000);
                    break;
                case RIGHT:
                case KP_RIGHT:
                    increaseVideoTime(5000);
                    break;
                case SPACE:
                case P:
                    changePlayPauseState();
                    break;
                case F11:
                    playerControls.toggleFullscreen();
                    break;
            }
        });
        playerPane.setOnKeyPressed(event -> {
            switch (event.getCode()) {
                case G:
                    updateSubtitleOffset(event, false);
                    break;
                case H:
                    updateSubtitleOffset(event, true);
                    break;
            }
        });

        idleTimer.setOnFinished(e -> onHideOverlay());
        playerPane.addEventHandler(Event.ANY, e -> onShowOverlay());
    }

    private void initializeSubtitleTrack() {
        SubtitleSettings subtitleSettings = settingsService.getSettings().getSubtitleSettings();

        log.trace("Setting subtitle track defaults to \"{}\"", subtitleSettings);
        subtitleTrack.setFontFamily(subtitleSettings.getFontFamily().getFamily());
        subtitleTrack.setFontSize(subtitleSettings.getFontSize());
        subtitleTrack.setFontWeight(getFontWeight(subtitleSettings.isBold()));
        subtitleTrack.setDecoration(subtitleSettings.getDecoration());

        subtitleSettings.addListener(evt -> {
            log.trace("Subtitle setting \"{}\" is being changed, updating subtitle track", evt.getPropertyName());
            switch (evt.getPropertyName()) {
                case SubtitleSettings.FONT_FAMILY_PROPERTY:
                    subtitleTrack.setFontFamily((String) evt.getNewValue());
                    break;
                case SubtitleSettings.FONT_SIZE_PROPERTY:
                    subtitleTrack.setFontSize((Integer) evt.getNewValue());
                    break;
                case SubtitleSettings.BOLD_PROPERTY:
                    var bold = (Boolean) evt.getNewValue();
                    subtitleTrack.setFontWeight(getFontWeight(bold));
                    break;
                case SubtitleSettings.DECORATION_PROPERTY:
                    subtitleTrack.setDecoration((DecorationType) evt.getNewValue());
                    break;
            }
        });

        offsetTimer.setOnFinished(event -> {
            FadeTransition fadeTransition = new FadeTransition(Duration.millis(INFO_FADE_DURATION), subtitleOffset);
            fadeTransition.setToValue(0);
            fadeTransition.play();
        });

        subtitleTrack.offsetProperty().addListener((observable, oldValue, newValue) -> {
            subtitleOffset.setText(localeText.get(VideoMessage.SUBTITLES_OFFSET, newValue.doubleValue()));
            subtitleOffset.setOpacity(1);
            offsetTimer.playFromStart();
        });
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
            // download the subtitle before starting the playback
            SubtitleInfo subtitle = activitySubtitle.get();
            onSubtitleChanged(subtitle, activity.getUrl());
        } else {
            // instant play video
            playUrl(activity.getUrl());
        }
    }

    private void onClose() {
        reset();

        torrentService.stopStream();
    }

    private void onSubtitleChanged(SubtitleInfo subtitle) {
        onSubtitleChanged(subtitle, null);
    }

    //TODO: Fix double invocation when starting a playback
    private void onSubtitleChanged(SubtitleInfo subtitle, String playbackUrl) {
        if (subtitle == null || subtitle.isNone()) {
            subtitleTrack.clear();
        } else {
            log.debug("Downloading subtitle \"{}\" for video playback", subtitle);

            subtitleService.downloadAndParse(subtitle).whenComplete((subtitles, throwable) -> {
                if (throwable != null) {
                    log.error("Video subtitle failed, " + throwable.getMessage(), throwable);
                } else {
                    log.debug("Successfully retrieved parsed subtitle");
                    subtitleTrack.setSubtitles(subtitles);
                }

                if (StringUtils.isNotEmpty(playbackUrl))
                    playUrl(playbackUrl);
            });
        }
    }

    private void onHideOverlay() {
        if (videoPlayer.getPlayerState() != PlayerState.PLAYING || playerHeader.isBlocked() || playerControls.isBlocked())
            return;

        log.trace("Hiding video player overlay");
        playerPane.setCursor(Cursor.NONE);
        playerVideoOverlay.setCursor(Cursor.NONE);

        FadeTransition transitionHeader = new FadeTransition(Duration.millis(OVERLAY_FADE_DURATION), playerHeaderPane);
        FadeTransition transitionControls = new FadeTransition(Duration.millis(OVERLAY_FADE_DURATION), playerControlsPane);

        transitionHeader.setToValue(0.0);
        transitionControls.setToValue(0.0);

        transitionHeader.play();
        transitionControls.play();
    }

    private void onShowOverlay() {
        playerPane.setCursor(Cursor.DEFAULT);
        playerVideoOverlay.setCursor(Cursor.HAND);

        playerHeaderPane.setOpacity(1.0);
        playerControlsPane.setOpacity(1.0);

        idleTimer.playFromStart();
    }

    private void onVideoStopped() {
        // check if the video has been started for more than 1.5 sec before exiting the video player
        // this should fix the issue of the video player closing directly in some cases
        if (System.currentTimeMillis() - videoChangeTime <= 30000)
            return;

        close();
    }

    private void playUrl(String url) {
        updateActiveVideoPlayer(url);
        this.url = url;
        this.videoPlayer.play(url);

        var filename = FilenameUtils.getName(url);

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
        if (videoPlayer == this.videoPlayer)
            return;

        // remove old video player listeners
        if (this.videoPlayer != null) {
            this.videoPlayer.playerStateProperty().removeListener(playerStateListener);
            this.videoPlayer.timeProperty().removeListener(timeListener);
            this.videoPlayer.durationProperty().removeListener(durationListener);
        }

        // add video player listeners to the new video player
        videoPlayer.playerStateProperty().addListener(playerStateListener);
        videoPlayer.timeProperty().addListener(timeListener);
        videoPlayer.durationProperty().addListener(durationListener);

        Platform.runLater(() -> {
            videoView.getChildren().clear();
            Node videoSurface = videoPlayer.getVideoSurface();

            if (videoSurface instanceof Canvas) {
                var canvas = (Canvas) videoSurface;
                canvas.widthProperty().bind(videoView.widthProperty());
                canvas.heightProperty().bind(videoView.heightProperty());
            } else if (videoSurface instanceof WebView) {
                var webview = (WebView) videoSurface;
                webview.prefWidthProperty().bind(videoView.widthProperty());
                webview.prefHeightProperty().bind(videoView.heightProperty());
            } else if (videoSurface instanceof MediaView) {
                var media = (MediaView) videoSurface;
                media.fitWidthProperty().bind(videoView.widthProperty());
                media.fitWidthProperty().bind(videoView.heightProperty());
            }

            videoView.getChildren().add(videoSurface);
        });

        this.videoPlayer = videoPlayer;
    }

    private void increaseVideoTime(long amount) {
        log.trace("Increasing video time with {}", amount);
        long newTime = videoPlayer.getTime() + amount;
        long duration = videoPlayer.getDuration();

        if (newTime > duration)
            newTime = duration;

        videoPlayer.seek(newTime);
    }

    private void reset() {
        log.trace("Video player component is being reset");
        this.url = null;
        this.media = null;
        this.quality = null;
        this.videoChangeTime = 0;
        this.subtitleTrack.setOffset(0.0);

        Platform.runLater(() -> {
            subtitleTrack.clear();
            errorText.setText("");
        });
        taskExecutor.execute(videoPlayer::stop);
    }

    private void updateSubtitleOffset(KeyEvent event, boolean increaseOffset) {
        double offset = 0.1;

        if (event.isControlDown() && event.isShiftDown()) {
            offset = 10.0;
        } else if (event.isControlDown()) {
            offset = 5.0;
        } else if (event.isShiftDown()) {
            offset = 1.0;
        }

        double currentOffset = subtitleTrack.getOffset();

        if (increaseOffset) {
            subtitleTrack.setOffset(currentOffset + offset);
        } else {
            subtitleTrack.setOffset(currentOffset - offset);
        }
    }

    private FontWeight getFontWeight(boolean isBold) {
        return isBold ? FontWeight.BOLD : FontWeight.NORMAL;
    }

    private void changePlayPauseState() {
        if (videoPlayer.getPlayerState() == PlayerState.PAUSED) {
            log.trace("Video player state is being changed to \"resume\"");
            videoPlayer.resume();
        } else {
            log.trace("Video player state is being changed to \"paused\"");
            videoPlayer.pause();
        }
    }

    private void close() {
        log.trace("Video player is being closed");
        // keep a copy of the information for later use in the activity
        var url = this.url;
        var media = this.media;
        var quality = this.quality;
        var time = playerControls.getTime();
        var duration = playerControls.getDuration();

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

        onClose();
    }

    @FXML
    private void onPlayerClick() {
        changePlayPauseState();
    }

    //endregion
}
