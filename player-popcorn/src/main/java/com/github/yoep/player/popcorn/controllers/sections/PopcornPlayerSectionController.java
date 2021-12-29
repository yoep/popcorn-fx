package com.github.yoep.player.popcorn.controllers.sections;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.player.popcorn.messages.VideoMessage;
import com.github.yoep.player.popcorn.services.PlaybackService;
import com.github.yoep.player.popcorn.services.SubtitleEventService;
import com.github.yoep.player.popcorn.services.VideoService;
import com.github.yoep.player.popcorn.subtitles.controls.SubtitleTrack;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.screen.ScreenService;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayer;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.backend.settings.models.subtitles.DecorationType;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import javafx.animation.FadeTransition;
import javafx.animation.PauseTransition;
import javafx.application.Platform;
import javafx.event.Event;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Cursor;
import javafx.scene.Node;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressIndicator;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import javafx.scene.layout.StackPane;
import javafx.scene.text.FontWeight;
import javafx.util.Duration;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@ViewController
@RequiredArgsConstructor
public class PopcornPlayerSectionController implements Initializable {
    private static final int OVERLAY_FADE_DURATION = 1500;
    private static final int INFO_FADE_DURATION = 2000;
    private static final String BUFFER_STYLE_CLASS = "buffer";

    private final PlaybackService playbackService;
    private final ScreenService screenService;
    private final SettingsService settingsService;
    private final VideoService videoService;
    private final SubtitleEventService popcornSubtitleService;
    private final LocaleText localeText;
    protected final PauseTransition idleTimer = getIdleTimer();
    protected final PauseTransition offsetTimer = getOffsetTimer();

    private FadeTransition fadeTransition;
    private FadeTransition transitionHeader;
    private FadeTransition transitionControls;
    private Pane bufferIndicator;
    private boolean uiBlocked;
    private boolean isPlaying;

    @FXML
    Pane playerPane;
    @FXML
    Pane videoView;
    @FXML
    Pane bufferPane;
    @FXML
    Label subtitleOffset;
    @FXML
    Label errorText;
    @FXML
    Pane playerVideoOverlay;
    @FXML
    Pane playerHeaderPane;
    @FXML
    Pane playerControlsPane;
    @FXML
    SubtitleTrack subtitleTrack;

    //region Methods

    /**
     * Update the video view with the given node.
     *
     * @param view The view node to use.
     */
    public void setVideoView(Node view) {
        videoView.getChildren().setAll(view);
    }

    public void updatePlaybackState(boolean isPlaying) {
        this.isPlaying = isPlaying;

        if (!isPlaying) {
            onShowOverlay(null);
        }
    }

    public void updateTime(Long time) {
        subtitleTrack.onTimeChanged(time);
    }

    public void reset() {
        Platform.runLater(() -> {
            subtitleTrack.clear();
            errorText.setText(null);
        });
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeFaders();
        initializeSceneEvents();
        initializeListeners();
        initializePaneListeners();
        initializeSubtitleTrack();
    }

    private void initializeSceneEvents() {
        idleTimer.setOnFinished(e -> onHideOverlay());
        playerPane.setOnKeyReleased(this::onPlayerKeyReleased);
        playerPane.addEventHandler(Event.ANY, this::onShowOverlay);
    }

    private void initializeListeners() {
        playbackService.addPlayerListener(createPlayerListener());
        popcornSubtitleService.activeSubtitleProperty().addListener((observableValue, subtitle, newSubtitle) -> onSubtitleChanged(newSubtitle));
    }

    private void initializePaneListeners() {
        playerHeaderPane.setOnMouseEntered(event -> uiBlocked = true);
        playerHeaderPane.setOnMouseExited(event -> uiBlocked = false);

        playerControlsPane.setOnMouseEntered(event -> uiBlocked = true);
        playerControlsPane.setOnMouseExited(event -> uiBlocked = false);
    }

    private void initializeSubtitleTrack() {
        var subtitleSettings = settingsService.getSettings().getSubtitleSettings();

        log.trace("Setting subtitle track defaults to \"{}\"", subtitleSettings);
        subtitleTrack.setFontFamily(subtitleSettings.getFontFamily().getFamily());
        subtitleTrack.setFontWeight(getFontWeight(subtitleSettings.isBold()));
        subtitleTrack.setDecoration(subtitleSettings.getDecoration());

        // bind the subtitle size to the video player service
        subtitleTrack.fontSizeProperty().bind(popcornSubtitleService.subtitleSizeProperty());

        subtitleSettings.addListener(evt -> {
            log.trace("Subtitle setting \"{}\" is being changed, updating subtitle track", evt.getPropertyName());
            switch (evt.getPropertyName()) {
                case SubtitleSettings.FONT_FAMILY_PROPERTY:
                    subtitleTrack.setFontFamily((String) evt.getNewValue());
                    break;
                case SubtitleSettings.FONT_SIZE_PROPERTY:
                    popcornSubtitleService.setSubtitleSize((Integer) evt.getNewValue());
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
            fadeTransition.setToValue(0);
            fadeTransition.playFromStart();
        });

        subtitleTrack.offsetProperty().addListener((observable, oldValue, newValue) -> Platform.runLater(() -> {
            subtitleOffset.setText(localeText.get(VideoMessage.SUBTITLES_OFFSET, newValue.doubleValue()));
            popcornSubtitleService.setSubtitleOffset(newValue.longValue() * 1000);
            fadeTransition.stop();
            subtitleOffset.setOpacity(1);
            offsetTimer.playFromStart();
        }));
    }

    private void initializeFaders() {
        fadeTransition = new FadeTransition(Duration.millis(INFO_FADE_DURATION), subtitleOffset);
    }

    //endregion

    //region Functions

    private void onBuffering() {
        updateBufferIndicator(true);
    }

    private void onPlaying() {
        updateBufferIndicator(false);
    }

    private void onError() {
        Platform.runLater(() -> errorText.setText(localeText.get(VideoMessage.VIDEO_ERROR)));
        updateBufferIndicator(false);
    }

    private PauseTransition getIdleTimer() {
        return new PauseTransition(Duration.seconds(3));
    }

    private PauseTransition getOffsetTimer() {
        return new PauseTransition(Duration.seconds(2));
    }

    private void updateBufferIndicator(boolean showBuffer) {
        // check if the buffer is already present and it should not be shown
        if (!showBuffer && bufferIndicator != null) {
            log.trace("Removing the buffer indicator from the player view");
            Platform.runLater(() -> {
                bufferPane.getChildren().clear();
                bufferIndicator = null;
            });
        } else if (showBuffer && bufferIndicator == null) {
            log.trace("Adding the buffer indicator to the player view");
            bufferIndicator = new StackPane();
            bufferIndicator.getStyleClass().add(BUFFER_STYLE_CLASS);
            bufferIndicator.getChildren().add(new ProgressIndicator());

            Platform.runLater(() -> bufferPane.getChildren().add(bufferIndicator));
        }
    }

    private void onHideOverlay() {
        if (uiBlocked || !isPlaying)
            return;

        log.trace("Hiding video player overlay");
        playerPane.setCursor(Cursor.NONE);
        playerVideoOverlay.setCursor(Cursor.NONE);

        transitionHeader = new FadeTransition(Duration.millis(OVERLAY_FADE_DURATION), playerHeaderPane);
        transitionControls = new FadeTransition(Duration.millis(OVERLAY_FADE_DURATION), playerControlsPane);

        transitionHeader.setToValue(0.0);
        transitionControls.setToValue(0.0);

        transitionHeader.play();
        transitionControls.play();
    }

    private void onShowOverlay(Event event) {
        // cancel the transition faders
        if (transitionHeader != null) {
            transitionHeader.stop();
        }
        if (transitionControls != null) {
            transitionControls.stop();
        }

        playerPane.setCursor(Cursor.DEFAULT);
        playerVideoOverlay.setCursor(Cursor.HAND);

        playerHeaderPane.setOpacity(1.0);
        playerControlsPane.setOpacity(1.0);

        idleTimer.playFromStart();
    }

    /**
     * Invoked when a key is being pressed inside the player section.
     *
     * @param event The key event that occurred.
     */
    private void onPlayerKeyReleased(KeyEvent event) {
        switch (event.getCode()) {
            case SPACE:
            case P:
                playbackService.togglePlayerPlaybackState();
                event.consume();
                break;
            case F11:
                screenService.toggleFullscreen();
                event.consume();
                break;
            case G:
                updateSubtitleOffset(event, false);
                event.consume();
                break;
            case H:
                updateSubtitleOffset(event, true);
                event.consume();
                break;
            case LEFT:
            case KP_LEFT:
                playbackService.videoTimeOffset(-5000);
                event.consume();
                break;
            case RIGHT:
            case KP_RIGHT:
                playbackService.videoTimeOffset(5000);
                event.consume();
                break;
        }
    }

    private void onSubtitleChanged(Subtitle subtitle) {
        var supportNativeSubtitlePlayback = videoService.getVideoPlayer()
                .map(VideoPlayer::supportsNativeSubtitleFile)
                .orElse(false);

        if (subtitle.isNone() || supportNativeSubtitlePlayback) {
            subtitleTrack.clear();
        } else {
            subtitleTrack.setSubtitle(subtitle);
        }
    }

    private FontWeight getFontWeight(boolean isBold) {
        return isBold ? FontWeight.BOLD : FontWeight.NORMAL;
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

    private PlayerListener createPlayerListener() {
        return new PlayerListener() {
            @Override
            public void onDurationChanged(long newDuration) {

            }

            @Override
            public void onTimeChanged(long newTime) {

            }

            @Override
            public void onStateChanged(PlayerState newState) {
                switch (newState) {
                    case BUFFERING:
                        onBuffering();
                        break;
                    case PLAYING:
                        onPlaying();
                        break;
                    case ERROR:
                        onError();
                        break;
                }
            }
        };
    }

    @FXML
    void onPlayerClick(MouseEvent event) {
        event.consume();
        playbackService.togglePlayerPlaybackState();
    }

    //endregion
}
