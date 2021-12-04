package com.github.yoep.player.popcorn.controllers.sections;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.player.adapter.listeners.PlayerListener;
import com.github.yoep.player.adapter.state.PlayerState;
import com.github.yoep.player.popcorn.services.PlaybackService;
import com.github.yoep.popcorn.ui.keepalive.KeepAliveService;
import com.github.yoep.popcorn.ui.messages.VideoMessage;
import com.github.yoep.popcorn.ui.view.services.FullscreenService;
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
    private final FullscreenService fullscreenService;
    private final LocaleText localeText;
    protected final PauseTransition idleTimer = getIdleTimer();
    protected final PauseTransition offsetTimer = getOffsetTimer();

    private FadeTransition fadeTransition;
    private Pane bufferIndicator;
    private boolean uiBlocked;

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

    //region Methods

    /**
     * Update the video view with the given node.
     *
     * @param view The view node to use.
     */
    public void setVideoView(Node view) {
        videoView.getChildren().setAll(view);
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeSceneEvents();
        initializeListeners();
    }

    private void initializeSceneEvents() {
        idleTimer.setOnFinished(e -> onHideOverlay());
        playerPane.setOnKeyReleased(this::onPlayerKeyReleased);
        playerPane.addEventHandler(Event.ANY, this::onShowOverlay);
    }

    private void initializeListeners() {
        playbackService.addPlayerListener(createPlayerListener());
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
        if (uiBlocked)
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

    private void onShowOverlay(Event event) {
        // verify if the event is a key event
        // if so, do some additional check before showing the overlay
        if (event instanceof KeyEvent) {
            var keyEvent = (KeyEvent) event;

            // verify that the key event is not the key used by the keep alive service
            // if so, don't show the overlay and ignore the event
            if (keyEvent.getCode() == KeepAliveService.SIGNAL)
                return;
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
                fullscreenService.toggle();
                event.consume();
                break;
            case G:
//                updateSubtitleOffset(event, false);
                event.consume();
                break;
            case H:
//                updateSubtitleOffset(event, true);
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
