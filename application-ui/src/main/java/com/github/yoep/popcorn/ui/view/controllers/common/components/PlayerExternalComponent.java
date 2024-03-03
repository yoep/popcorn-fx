package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.player.PlayerAction;
import com.github.yoep.popcorn.backend.utils.TimeUtils;
import com.github.yoep.popcorn.ui.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.ui.view.controls.ProgressControl;
import com.github.yoep.popcorn.ui.view.listeners.PlayerExternalListener;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.popcorn.ui.view.services.PlayerExternalComponentService;
import javafx.application.Platform;
import javafx.event.EventHandler;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.GridPane;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@ViewController
@RequiredArgsConstructor
public class PlayerExternalComponent implements Initializable {
    private final ImageService imageService;
    private final PlayerExternalComponentService playerExternalService;
    private final ViewLoader viewLoader;
    private final EventHandler<KeyEvent> keyPressedEventHandler = this::onPaneKeyReleased;
    final ProgressInfoComponent infoComponent = new ProgressInfoComponent();

    @FXML
    Pane playerExternalPane;
    @FXML
    Pane dataPane;
    @FXML
    BackgroundImageCover backgroundImage;
    @FXML
    Label titleText;
    @FXML
    Label captionText;
    @FXML
    Label timeText;
    @FXML
    Label durationText;
    @FXML
    ProgressControl playbackProgress;
    @FXML
    Icon playPauseIcon;
    @FXML
    Pane progressInfoPane;

    //region Init

    @PostConstruct
    void init() {
        playerExternalService.addListener(new PlayerExternalListener() {
            @Override
            public void onRequestChanged(PlayRequest request) {
                PlayerExternalComponent.this.onRequestChanged(request);
            }

            @Override
            public void onTimeChanged(long time) {
                onPlayerTimeChanged(time);
            }

            @Override
            public void onDurationChanged(long duration) {
                onPlayerDurationChanged(duration);
            }

            @Override
            public void onStateChanged(PlayerState state) {
                onPlayerStateChanged(state);
            }

            @Override
            public void onDownloadStatus(DownloadStatus status) {
                PlayerExternalComponent.this.onDownloadStatus(status);
            }
        });
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeProgressInfo();
        playerExternalPane.sceneProperty().addListener((observableValue, scene, newScene) -> {
            if (newScene != null) {
                log.trace("Registering key event handler to scene for external player");
                newScene.addEventHandler(KeyEvent.KEY_RELEASED, keyPressedEventHandler);
            } else {
                log.trace("Removing key event handler from scene for external player");
                scene.removeEventHandler(KeyEvent.KEY_RELEASED, keyPressedEventHandler);
            }
        });
    }

    //endregion

    //region Functions

    private void initializeProgressInfo() {
        dataPane.getChildren().remove(progressInfoPane);
        progressInfoPane = viewLoader.load("common/components/progress-info.component.fxml", infoComponent);
        GridPane.setColumnIndex(progressInfoPane, 1);
        GridPane.setRowIndex(progressInfoPane, 3);
        dataPane.getChildren().add(progressInfoPane);
    }

    private void onRequestChanged(PlayRequest request) {
        reset();
        Platform.runLater(() -> {
            titleText.setText(request.getTitle());
            captionText.setText(request.getCaption().orElse(null));
        });
        request.getBackground()
                .ifPresent(this::loadBackgroundImage);
    }

    private void reset() {
        Platform.runLater(() -> {
            backgroundImage.reset();
            playbackProgress.setTime(0);
            playbackProgress.setDuration(0);
            playbackProgress.setLoadProgress(0.0);
        });
    }

    private void loadBackgroundImage(String url) {
        log.debug("Loading external player background url {}", url);
        imageService.load(url).whenComplete((image, throwable) -> {
            if (throwable == null) {
                Platform.runLater(() -> backgroundImage.setBackgroundImage(image));
            } else {
                log.error(throwable.getMessage(), throwable);
            }
        });
    }

    private void onPlayerDurationChanged(long duration) {
        Platform.runLater(() -> {
            playbackProgress.setDuration(duration);
            durationText.setText(TimeUtils.format(duration));
        });
    }

    private void onPlayerTimeChanged(long time) {
        Platform.runLater(() -> {
            timeText.setText(TimeUtils.format(time));
            playbackProgress.setTime(time);
        });
    }

    private void onPlayerStateChanged(PlayerState state) {
        switch (state) {
            case PLAYING -> updatePlayState(true);
            case PAUSED -> updatePlayState(false);
        }
    }

    private void updatePlayState(boolean isPlaying) {
        Platform.runLater(() -> {
            if (isPlaying) {
                playPauseIcon.setText(Icon.PAUSE_UNICODE);
            } else {
                playPauseIcon.setText(Icon.PLAY_UNICODE);
            }
        });
    }

    private void onDownloadStatus(DownloadStatus status) {
        Platform.runLater(() -> {
            playbackProgress.setLoadProgress(status.progress());
            infoComponent.update(status);
        });
    }

    void onPaneKeyReleased(KeyEvent event) {
        PlayerAction.FromKey(event.getCode()).ifPresent(e -> {
            switch (e) {
                case TOGGLE_PLAYBACK_STATE -> {
                    event.consume();
                    playerExternalService.togglePlaybackState();
                }
                case REVERSE -> {
                    event.consume();
                    playerExternalService.goBack();
                }
                case FORWARD -> {
                    event.consume();
                    playerExternalService.goForward();
                }
            }
        });
    }

    @FXML
    void onPlayPauseClicked(MouseEvent event) {
        event.consume();
        playerExternalService.togglePlaybackState();
    }

    @FXML
    void onStopClicked(MouseEvent event) {
        event.consume();
        playerExternalService.closePlayer();
    }

    @FXML
    void onGoBackClicked(MouseEvent event) {
        event.consume();
        playerExternalService.goBack();
    }

    @FXML
    void onGoForwardClicked(MouseEvent event) {
        event.consume();
        playerExternalService.goForward();
    }

    //endregion
}
