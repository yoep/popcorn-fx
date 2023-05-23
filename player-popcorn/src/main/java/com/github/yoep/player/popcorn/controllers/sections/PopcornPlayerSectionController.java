package com.github.yoep.player.popcorn.controllers.sections;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.player.popcorn.listeners.PopcornPlayerSectionListener;
import com.github.yoep.player.popcorn.messages.VideoMessage;
import com.github.yoep.player.popcorn.services.PopcornPlayerSectionService;
import com.github.yoep.player.popcorn.services.SubtitleManagerService;
import com.github.yoep.player.popcorn.subtitles.controls.SubtitleTrack;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayVideoEvent;
import com.github.yoep.popcorn.backend.events.PlayerStoppedEvent;
import com.github.yoep.popcorn.backend.player.PlayerAction;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.subtitles.DecorationType;
import com.github.yoep.popcorn.backend.subtitles.Subtitle;
import com.github.yoep.popcorn.ui.events.SubtitleOffsetEvent;
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
import javafx.scene.input.ScrollEvent;
import javafx.scene.layout.AnchorPane;
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
    static final int OVERLAY_FADE_DURATION = 1500;
    static final int INFO_FADE_DURATION = 2000;
    static final int VOLUME_INCREASE_AMOUNT = 5;
    static final double INFO_TOP_TV_MODE = 100d;
    static final String BUFFER_STYLE_CLASS = "buffer";
    static final String VIEW_CONTROLS = "components/player-controls.component.fxml";

    private final PopcornPlayerSectionService sectionService;
    private final SubtitleManagerService subtitleManagerService;
    private final LocaleText localeText;
    private final EventPublisher eventPublisher;
    private final ViewLoader viewLoader;
    private final ApplicationConfig applicationConfig;

    final PauseTransition idleTimer = new PauseTransition(Duration.seconds(3));
    final PauseTransition offsetTimer = new PauseTransition(Duration.seconds(2));

    private FadeTransition fadeTransition;
    private FadeTransition transitionHeader;
    private FadeTransition transitionControls;
    private Pane bufferIndicator;
    private PlayerState lastKnownPlayerState;
    private boolean uiBlocked;

    @FXML
    AnchorPane playerPane;
    @FXML
    Pane videoView;
    @FXML
    Pane bufferPane;
    @FXML
    Label infoLabel;
    @FXML
    Label errorText;
    @FXML
    Pane playerVideoOverlay;
    @FXML
    Pane playerHeaderPane;
    Pane playerControlsPane;
    @FXML
    SubtitleTrack subtitleTrack;

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeFaders();
        initializeSceneEvents();
        initializeMode();
        initializeListeners();
        initializePaneListeners();
        initializeSubtitleTrack();
    }

    private void initializeSceneEvents() {
        idleTimer.setOnFinished(e -> onHideOverlay());
        playerPane.setOnKeyReleased(this::onPlayerKeyReleased);
        playerPane.setOnScroll(this::onPlayerScrolled);
        playerPane.addEventHandler(Event.ANY, this::onShowOverlay);
    }

    private void initializeMode() {
        playerControlsPane = viewLoader.load(VIEW_CONTROLS);
        AnchorPane.setLeftAnchor(playerControlsPane, 0d);
        AnchorPane.setRightAnchor(playerControlsPane, 0d);
        AnchorPane.setBottomAnchor(playerControlsPane, applicationConfig.isTvMode() ? 150d : 0d);
        playerPane.getChildren().add(playerControlsPane);

        AnchorPane.setTopAnchor(infoLabel, applicationConfig.isTvMode() ? INFO_TOP_TV_MODE : 50d);
    }

    private void initializeListeners() {
        eventPublisher.register(PlayVideoEvent.class, event -> {
            Platform.runLater(() -> errorText.setText(""));
            return event;
        }, EventPublisher.HIGHEST_ORDER);
        eventPublisher.register(PlayerStoppedEvent.class, event -> {
            Platform.runLater(() -> {
                subtitleTrack.clear();
                errorText.setText(null);
            });
            return event;
        });
        eventPublisher.register(SubtitleOffsetEvent.class, event -> {
            Platform.runLater(() -> onSubtitleOffsetChanged(event.getOffset()));
            return event;
        });
        sectionService.addListener(new PopcornPlayerSectionListener() {
            @Override
            public void onSubtitleChanged(Subtitle subtitle) {
                PopcornPlayerSectionController.this.onSubtitleChanged(subtitle);
            }

            @Override
            public void onSubtitleDisabled() {
                PopcornPlayerSectionController.this.onSubtitleDisabled();
            }

            @Override
            public void onPlayerStateChanged(PlayerState state) {
                PopcornPlayerSectionController.this.onPlayerStateChanged(state);
            }

            @Override
            public void onPlayerTimeChanged(long time) {
                PopcornPlayerSectionController.this.onPlayerTimeChanged(time);
            }

            @Override
            public void onSubtitleSizeChanged(int fontSize) {
                PopcornPlayerSectionController.this.onSubtitleFontSizeChanged(fontSize);
            }

            @Override
            public void onSubtitleFamilyChanged(String newFontFamily) {
                PopcornPlayerSectionController.this.onSubtitleFamilyChanged(newFontFamily);
            }

            @Override
            public void onSubtitleFontWeightChanged(Boolean bold) {
                PopcornPlayerSectionController.this.onSubtitleFontWeightChanged(bold);
            }

            @Override
            public void onSubtitleDecorationChanged(DecorationType newDecorationType) {
                PopcornPlayerSectionController.this.onSubtitleDecorationChanged(newDecorationType);
            }

            @Override
            public void onVideoViewChanged(Node videoView) {
                PopcornPlayerSectionController.this.onVideoViewChanged(videoView);
            }

            @Override
            public void onVolumeChanged(int volume) {
                PopcornPlayerSectionController.this.onVolumeChanged(volume);
            }
        });
    }

    private void initializePaneListeners() {
        // ignore all mouse events if the mouse is disabled
        if (applicationConfig.isMouseDisabled())
            return;

        playerHeaderPane.setOnMouseEntered(event -> uiBlocked = true);
        playerHeaderPane.setOnMouseExited(event -> uiBlocked = false);

        playerControlsPane.setOnMouseEntered(event -> uiBlocked = true);
        playerControlsPane.setOnMouseExited(event -> uiBlocked = false);
    }

    private void initializeSubtitleTrack() {
        offsetTimer.setOnFinished(event -> {
            fadeTransition.setToValue(0);
            fadeTransition.playFromStart();
        });

        subtitleTrack.offsetProperty().addListener((observable, oldValue, newValue) -> Platform.runLater(() -> {
            subtitleManagerService.updateSubtitleOffset(newValue.intValue() * 1000);
            showInfo(localeText.get(VideoMessage.SUBTITLES_OFFSET, newValue.doubleValue()));
        }));

        sectionService.provideSubtitleValues();
    }

    private void initializeFaders() {
        fadeTransition = new FadeTransition(Duration.millis(INFO_FADE_DURATION), infoLabel);
    }

    //endregion

    //region Functions

    private void onBuffering() {
        updateBufferIndicator(true);
    }

    private void onPlaying() {
        updateBufferIndicator(false);
        idleTimer.play();
    }

    private void onError() {
        log.warn("Video player entered ERROR state");
        Platform.runLater(() -> errorText.setText(localeText.get(VideoMessage.VIDEO_ERROR)));
        updateBufferIndicator(false);
    }

    private void onPlayerStateChanged(PlayerState newState) {
        this.lastKnownPlayerState = newState;

        switch (newState) {
            case BUFFERING -> onBuffering();
            case PLAYING -> onPlaying();
            case PAUSED -> onShowOverlay(null);
            case ERROR -> {
                onError();
                onShowOverlay(null);
            }
        }
    }

    private void onPlayerTimeChanged(long time) {
        subtitleTrack.onTimeChanged(time);
    }

    private void onSubtitleFontSizeChanged(int newFontSize) {
        subtitleTrack.setFontSize(newFontSize);
    }

    private void onSubtitleDecorationChanged(DecorationType newDecoration) {
        subtitleTrack.setDecoration(newDecoration);
    }

    private void onSubtitleFamilyChanged(String newFontFamily) {
        subtitleTrack.setFontFamily(newFontFamily);
    }

    private void onSubtitleFontWeightChanged(boolean isBold) {
        subtitleTrack.setFontWeight(getFontWeight(isBold));
    }

    private void onVideoViewChanged(Node view) {
        videoView.getChildren().setAll(view);
    }

    private void onVolumeChanged(int volume) {
        showInfo(localeText.get(VideoMessage.VIDEO_VOLUME, volume));
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
        if (uiBlocked || lastKnownPlayerState != PlayerState.PLAYING)
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
        PlayerAction.FromKey(event.getCode()).ifPresent(e -> {
            switch (e) {
                case TOGGLE_PLAYBACK_STATE -> {
                    event.consume();
                    sectionService.togglePlayerPlaybackState();
                }
                case TOGGLE_FULLSCREEN -> {
                    event.consume();
                    sectionService.toggleFullscreen();
                }
                case DECREASE_SUBTITLE_OFFSET -> {
                    event.consume();
                    handleSubtitleOffsetKeyPressed(event, false);
                }
                case INCREASE_SUBTITLE_OFFSET -> {
                    event.consume();
                    handleSubtitleOffsetKeyPressed(event, true);
                }
                case REVERSE -> {
                    if (!applicationConfig.isTvMode()) {
                        event.consume();
                        sectionService.videoTimeOffset(-5000);
                    }
                }
                case FORWARD -> {
                    if (!applicationConfig.isTvMode()) {
                        event.consume();
                        sectionService.videoTimeOffset(5000);
                    }
                }
            }
        });
    }

    private void onPlayerScrolled(ScrollEvent event) {
        event.consume();
        var volumeDelta = VOLUME_INCREASE_AMOUNT;

        if (event.getDeltaY() < 0) {
            volumeDelta = -volumeDelta;
        }

        sectionService.onVolumeScroll(volumeDelta);
    }

    private void onSubtitleChanged(Subtitle subtitle) {
        var supportNativeSubtitlePlayback = sectionService.isNativeSubtitlePlaybackSupported();

        if (subtitle.isNone() || supportNativeSubtitlePlayback) {
            subtitleTrack.clear();
        } else {
            subtitleTrack.setSubtitle(subtitle);
        }
    }

    private void onSubtitleDisabled() {
        subtitleTrack.clear();
    }

    private FontWeight getFontWeight(boolean isBold) {
        return isBold ? FontWeight.BOLD : FontWeight.NORMAL;
    }

    private void handleSubtitleOffsetKeyPressed(KeyEvent event, boolean increaseOffset) {
        double offset = 0.1;

        if (event.isControlDown() && event.isShiftDown()) {
            offset = 10.0;
        } else if (event.isControlDown()) {
            offset = 5.0;
        } else if (event.isShiftDown()) {
            offset = 1.0;
        }

        onSubtitleOffsetChanged(increaseOffset ? offset : -offset);
    }

    private void onSubtitleOffsetChanged(double subtitleOffset) {
        var currentOffset = subtitleTrack.getOffset();
        subtitleTrack.setOffset(currentOffset + subtitleOffset);
    }

    private void showInfo(String message) {
        Platform.runLater(() -> {
            infoLabel.setText(message);
            fadeTransition.stop();
            infoLabel.setOpacity(1);
            offsetTimer.playFromStart();
        });
    }

    @FXML
    void onPlayerClick(MouseEvent event) {
        event.consume();
        sectionService.togglePlayerPlaybackState();
    }

    //endregion
}
