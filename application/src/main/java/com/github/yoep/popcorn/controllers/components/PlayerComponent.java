package com.github.yoep.popcorn.controllers.components;

import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.ClosePlayerActivity;
import com.github.yoep.popcorn.activities.PlayVideoActivity;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.subtitle.SubtitleService;
import com.github.yoep.popcorn.subtitle.controls.SubtitleTrack;
import com.github.yoep.popcorn.subtitle.models.DecorationType;
import com.github.yoep.popcorn.subtitle.models.SubtitleInfo;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.state.PlayerState;
import javafx.animation.FadeTransition;
import javafx.animation.PauseTransition;
import javafx.application.Platform;
import javafx.event.Event;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Cursor;
import javafx.scene.layout.Pane;
import javafx.scene.text.FontWeight;
import javafx.util.Duration;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;
import java.net.URL;
import java.util.Optional;
import java.util.ResourceBundle;

@Slf4j
@Component
@RequiredArgsConstructor
public class PlayerComponent implements Initializable {
    private static final int OVERLAY_FADE_DURATION = 1500;

    private final PauseTransition idleTimer = new PauseTransition(Duration.seconds(3));
    private final ActivityManager activityManager;
    private final TaskExecutor taskExecutor;
    private final TorrentService torrentService;
    private final SubtitleService subtitleService;
    private final SettingsService settingsService;
    private final PlayerHeaderComponent playerHeader;
    private final PlayerControlsComponent playerControls;
    private final VideoPlayer videoPlayer;

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
    private SubtitleTrack subtitleTrack;

    //region Methods

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        log.trace("Initializing video player component for JavaFX");
        initializeSceneEvents();
        initializeVideoPlayer();
        initializeSubtitleTrack();
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializeListeners();
    }

    private void initializeListeners() {
        activityManager.register(PlayVideoActivity.class, this::onPlayVideo);
        playerHeader.addListener(this::close);
        playerControls.addListener(new PlayerControlsListener() {
            @Override
            public void onSubtitleChanged(SubtitleInfo subtitle) {
                PlayerComponent.this.onSubtitleChanged(subtitle);
            }

            @Override
            public void onSubtitleSizeChanged(int pixelChange) {
                subtitleTrack.setFontSize(subtitleTrack.getFontSize() + pixelChange);
            }
        });
    }

    //endregion

    //region PreDestroy

    @PreDestroy
    private void dispose() {
        videoPlayer.dispose();
    }

    //endregion

    //region Functions

    private void initializeSceneEvents() {
        playerPane.setOnKeyReleased(event -> {
            switch (event.getCode()) {
                case LEFT:
                case KP_LEFT:
                    playerControls.increaseVideoTime(-5000);
                    break;
                case RIGHT:
                case KP_RIGHT:
                    playerControls.increaseVideoTime(5000);
                    break;
                case SPACE:
                case P:
                    playerControls.changePlayPauseState();
                    break;
                case F11:
                    playerControls.toggleFullscreen();
                    break;
            }
        });
        idleTimer.setOnFinished(e -> onHideOverlay());
        playerPane.addEventHandler(Event.ANY, e -> onShowOverlay());
    }

    private void initializeVideoPlayer() {
        videoPlayer.initialize(videoView);
        videoPlayer.playerStateProperty().addListener((observable, oldValue, newValue) -> {
            log.debug("Video player state changed to {}", newValue);

            switch (newValue) {
                case FINISHED:
                    break;
                case STOPPED:
                    onVideoStopped();
                    break;
            }
        });
        videoPlayer.timeProperty().addListener((observable, oldValue, newValue) -> subtitleTrack.onTimeChanged(newValue.longValue()));
    }

    private void initializeSubtitleTrack() {
        SubtitleSettings subtitleSettings = settingsService.getSettings().getSubtitleSettings();

        subtitleTrack.setFontFamily(subtitleSettings.getFontFamily().getFamily());
        subtitleTrack.setFontSize(subtitleSettings.getFontSize());
        subtitleTrack.setFontWeight(getFontWeight(subtitleSettings.isBold()));
        subtitleTrack.setDecoration(subtitleSettings.getDecoration());

        subtitleSettings.addListener(evt -> {
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
    }

    private void onPlayVideo(PlayVideoActivity activity) {
        log.debug("Received play video activity for url {}, quality {} and media {}", activity.getUrl(), activity.getQuality().orElse("-"),
                activity.getMedia());
        this.media = activity.getMedia();
        this.quality = activity.getQuality().orElse(null);
        this.videoChangeTime = System.currentTimeMillis();
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
        if (videoPlayer.getPlayerState() != PlayerState.PLAYING || playerHeader.isStreamInfoShowing())
            return;

        log.debug("Hiding video player overlay");
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
        if (System.currentTimeMillis() - videoChangeTime <= 1500)
            return;

        close();
    }

    private void playUrl(String url) {
        this.videoPlayer.play(url);
    }

    private void reset() {
        log.trace("Video player component is being reset");
        this.media = null;
        this.quality = null;
        this.videoChangeTime = 0;

        Platform.runLater(() -> subtitleTrack.clear());
        taskExecutor.execute(videoPlayer::stop);
    }

    private FontWeight getFontWeight(boolean isBold) {
        return isBold ? FontWeight.BOLD : FontWeight.NORMAL;
    }

    /**
     * Close the video player.
     * This will create a {@link ClosePlayerActivity} with the last known information about the video player state.
     */
    private void close() {
        log.trace("Video player is being closed");
        activityManager.register(new ClosePlayerActivity() {
            @Override
            public Media getMedia() {
                return media;
            }

            @Override
            public Optional<String> getQuality() {
                return Optional.ofNullable(quality);
            }

            @Override
            public long getTime() {
                return Optional.ofNullable(playerControls.getTime())
                        .orElse(UNKNOWN);
            }

            @Override
            public long getDuration() {
                return Optional.ofNullable(playerControls.getDuration())
                        .orElse(UNKNOWN);
            }
        });

        onClose();
    }

    @FXML
    private void onPlayerClick() {
        playerControls.changePlayPauseState();
    }

    //endregion
}
