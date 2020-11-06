package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.events.ClosePlayerEvent;
import com.github.yoep.popcorn.ui.messages.VideoMessage;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.ui.subtitles.Subtitle;
import com.github.yoep.popcorn.ui.subtitles.controls.SubtitleTrack;
import com.github.yoep.popcorn.ui.subtitles.models.DecorationType;
import com.github.yoep.popcorn.ui.view.services.VideoPlayerService;
import com.github.yoep.video.adapter.VideoPlayer;
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
import javafx.scene.control.ProgressIndicator;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.Pane;
import javafx.scene.layout.StackPane;
import javafx.scene.text.FontWeight;
import javafx.scene.web.WebView;
import javafx.util.Duration;
import lombok.AccessLevel;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.event.EventListener;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor(access = AccessLevel.PROTECTED)
public abstract class AbstractPlayerSectionController implements Initializable {
    private static final int OVERLAY_FADE_DURATION = 1500;
    private static final int INFO_FADE_DURATION = 2000;
    private static final String BUFFER_STYLE_CLASS = "buffer";

    protected final SettingsService settingsService;
    protected final VideoPlayerService videoPlayerService;
    protected final LocaleText localeText;

    protected final PauseTransition idleTimer = getIdleTimer();
    protected final PauseTransition offsetTimer = getOffsetTimer();
    protected final ChangeListener<PlayerState> playerStateListener = (observable, oldValue, newValue) -> onPlayerStateChanged(newValue);
    protected final ChangeListener<Number> timeListener = (observable, oldValue, newValue) -> onTimeChanged(newValue);

    protected Pane bufferIndicator;
    protected boolean uiBlocked;

    @FXML
    protected Pane playerPane;
    @FXML
    protected Pane bufferPane;
    @FXML
    protected Pane videoView;
    @FXML
    protected Label errorText;
    @FXML
    protected Label subtitleOffset;
    @FXML
    protected Pane playerHeaderPane;
    @FXML
    protected Pane playerVideoOverlay;
    @FXML
    protected Pane playerControlsPane;
    @FXML
    protected SubtitleTrack subtitleTrack;

    //region Methods

    @EventListener(ClosePlayerEvent.class)
    public void onClosePlayer() {
        onClose();
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        log.trace("Initializing video player component for JavaFX");
        initializeSceneEvents();
        initializeSubtitleTrack();
        initializePaneListeners();
    }

    private void initializeSceneEvents() {
        idleTimer.setOnFinished(e -> onHideOverlay());
        playerPane.setOnKeyReleased(this::onPlayerKeyReleased);
        playerPane.addEventHandler(Event.ANY, this::onShowOverlay);
    }

    private void initializeSubtitleTrack() {
        SubtitleSettings subtitleSettings = settingsService.getSettings().getSubtitleSettings();

        log.trace("Setting subtitle track defaults to \"{}\"", subtitleSettings);
        subtitleTrack.setFontFamily(subtitleSettings.getFontFamily().getFamily());
        subtitleTrack.setFontWeight(getFontWeight(subtitleSettings.isBold()));
        subtitleTrack.setDecoration(subtitleSettings.getDecoration());

        // bind the subtitle size to the video player service
        subtitleTrack.fontSizeProperty().bind(videoPlayerService.subtitleSizeProperty());

        subtitleSettings.addListener(evt -> {
            log.trace("Subtitle setting \"{}\" is being changed, updating subtitle track", evt.getPropertyName());
            switch (evt.getPropertyName()) {
                case SubtitleSettings.FONT_FAMILY_PROPERTY:
                    subtitleTrack.setFontFamily((String) evt.getNewValue());
                    break;
                case SubtitleSettings.FONT_SIZE_PROPERTY:
                    videoPlayerService.setSubtitleSize((Integer) evt.getNewValue());
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

        subtitleTrack.offsetProperty().addListener((observable, oldValue, newValue) -> Platform.runLater(() -> {
            subtitleOffset.setText(localeText.get(VideoMessage.SUBTITLES_OFFSET, newValue.doubleValue()));
            subtitleOffset.setOpacity(1);
            offsetTimer.playFromStart();
        }));
    }

    private void initializePaneListeners() {
        playerHeaderPane.setOnMouseEntered(event -> uiBlocked = true);
        playerHeaderPane.setOnMouseExited(event -> uiBlocked = false);

        playerControlsPane.setOnMouseEntered(event -> uiBlocked = true);
        playerControlsPane.setOnMouseExited(event -> uiBlocked = false);
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        log.trace("Initializing video player component for Spring");
        initializeVideoListeners();
    }

    private void initializeVideoListeners() {
        videoPlayerService.videoPlayerProperty().addListener((observable, oldValue, newValue) -> {
            if (oldValue != null) {
                oldValue.playerStateProperty().removeListener(playerStateListener);
                oldValue.timeProperty().removeListener(timeListener);
            }

            newValue.playerStateProperty().addListener(playerStateListener);
            newValue.timeProperty().addListener(timeListener);

            onVideoPlayerChanged(newValue);
        });
        videoPlayerService.subtitleProperty().addListener((observable, oldValue, newValue) -> onSubtitleChanged(newValue));
    }

    //endregion

    //region Functions

    /**
     * Get the idle timer which must be used to hide the header and controls.
     *
     * @return Returns the idle timer.
     */
    protected abstract PauseTransition getIdleTimer();

    /**
     * Get the offset timer which show/hides the subtitle offset info.
     *
     * @return Returns the subtitle offset timer.
     */
    protected abstract PauseTransition getOffsetTimer();

    /**
     * Invoked when the overlay is being hidden.
     */
    protected void onHideOverlay() {
        if (uiBlocked)
            return;

        var videoPlayer = videoPlayerService.getVideoPlayer();

        if (videoPlayer == null || videoPlayer.getPlayerState() != PlayerState.PLAYING)
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

    /**
     * Invoked when a key is being pressed inside the player section.
     *
     * @param event The key event that occurred.
     */
    protected void onPlayerKeyReleased(KeyEvent event) {
        switch (event.getCode()) {
            case SPACE:
            case P:
                videoPlayerService.changePlayPauseState();
                event.consume();
                break;
            case F11:
                videoPlayerService.toggleFullscreen();
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
        }
    }

    /**
     * Reset the controller.
     */
    protected void reset() {
        log.trace("Video player section controller is being reset");
        this.subtitleTrack.setOffset(0.0);

        Platform.runLater(() -> {
            subtitleTrack.clear();
            errorText.setText("");
        });
    }

    private void onVideoPlayerChanged(final VideoPlayer newValue) {
        Platform.runLater(() -> {
            videoView.getChildren().clear();
            Node videoSurface = newValue.getVideoSurface();

            if (videoSurface instanceof Canvas) {
                var canvas = (Canvas) videoSurface;
                canvas.widthProperty().bind(videoView.widthProperty());
                canvas.heightProperty().bind(videoView.heightProperty());
            } else if (videoSurface instanceof WebView) {
                var webview = (WebView) videoSurface;
                webview.prefWidthProperty().bind(videoView.widthProperty());
                webview.prefHeightProperty().bind(videoView.heightProperty());
            } else if (videoSurface instanceof StackPane) {
                var pane = (StackPane) videoSurface;

                pane.prefWidthProperty().bind(videoView.widthProperty());
                pane.prefHeightProperty().bind(videoView.heightProperty());
            }

            videoView.getChildren().add(videoSurface);
        });
    }

    private void onPlayerStateChanged(PlayerState newValue) {
        if (newValue != PlayerState.ERROR)
            log.debug("Video player state changed to {}", newValue);

        updateBufferIndicator(newValue == PlayerState.BUFFERING);

        switch (newValue) {
            case PLAYING:
                // start the idle timer when the video starts playing
                // no user feedback might be present when the video starts playing
                // causing the player overlay to never hide
                idleTimer.playFromStart();
                break;
            case ERROR:
                log.error("Video player state changed to {}", newValue);
                var error = videoPlayerService.getError();

                if (error != null)
                    log.error(error.getMessage(), error);

                Platform.runLater(() -> errorText.setText(localeText.get(VideoMessage.VIDEO_ERROR)));
                break;
        }
    }

    private void onTimeChanged(Number newValue) {
        subtitleTrack.onTimeChanged(newValue.longValue());
    }

    private void onClose() {
        reset();
    }

    private void onSubtitleChanged(Subtitle subtitle) {
        if (subtitle.isNone()) {
            subtitleTrack.clear();
        } else {
            subtitleTrack.setSubtitle(subtitle);
        }
    }

    private void onShowOverlay(Event event) {
        // verify if the event is a key event
        // if so, do some additional check before showing the overlay
        if (event instanceof KeyEvent) {
            var keyEvent = (KeyEvent) event;

            // verify that the key event is not the key used by the keep alive service
            // if so, don't show the overlay and ignore the event
            if (keyEvent.getCode() == KeyCode.ALT)
                return;
        }

        playerPane.setCursor(Cursor.DEFAULT);
        playerVideoOverlay.setCursor(Cursor.HAND);

        playerHeaderPane.setOpacity(1.0);
        playerControlsPane.setOpacity(1.0);

        idleTimer.playFromStart();
    }

    private FontWeight getFontWeight(boolean isBold) {
        return isBold ? FontWeight.BOLD : FontWeight.NORMAL;
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

    //endregion
}
