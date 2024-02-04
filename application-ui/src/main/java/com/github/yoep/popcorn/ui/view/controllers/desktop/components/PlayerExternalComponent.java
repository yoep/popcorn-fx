package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.player.PlayerAction;
import com.github.yoep.popcorn.backend.utils.TimeUtils;
import com.github.yoep.popcorn.ui.utils.ProgressUtils;
import com.github.yoep.popcorn.ui.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.ui.view.listeners.PlayerExternalListener;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.popcorn.ui.view.services.PlayerExternalComponentService;
import javafx.application.Platform;
import javafx.event.EventHandler;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@ViewController
@RequiredArgsConstructor
public class PlayerExternalComponent implements Initializable {
    private final ImageService imageService;
    private final PlayerExternalComponentService playerExternalService;
    private final EventHandler<KeyEvent> keyPressedEventHandler = this::onPaneKeyReleased;

    private Long duration;

    @FXML
    Pane playerExternalPane;
    @FXML
    BackgroundImageCover backgroundImage;
    @FXML
    Label titleText;
    @FXML
    Label timeText;
    @FXML
    Label durationText;
    @FXML
    ProgressBar playbackProgress;
    @FXML
    Icon playPauseIcon;
    @FXML
    Label progressPercentage;
    @FXML
    Label downloadText;
    @FXML
    Label uploadText;
    @FXML
    Label activePeersText;

    //region Init

    @PostConstruct
    void init() {
        playerExternalService.addListener(new PlayerExternalListener() {
            @Override
            public void onTitleChanged(String title) {
                PlayerExternalComponent.this.onTitleChanged(title);
            }

            @Override
            public void onMediaChanged(Media media) {
                PlayerExternalComponent.this.onMediaChanged(media);
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

    private void onTitleChanged(String title) {
        Platform.runLater(() -> titleText.setText(title));
    }

    private void onMediaChanged(Media media) {
        reset();
        Optional.ofNullable(media).ifPresent(this::loadBackgroundImage);
    }

    private void reset() {
        Platform.runLater(() -> {
            backgroundImage.reset();
            playbackProgress.setProgress(ProgressBar.INDETERMINATE_PROGRESS);
        });
    }

    private void loadBackgroundImage(Media media) {
        imageService.loadFanart(media).whenComplete((bytes, throwable) -> {
            if (throwable == null) {
                bytes.ifPresent(e -> backgroundImage.setBackgroundImage(e));
            } else {
                log.error(throwable.getMessage(), throwable);
            }
        });
    }

    private void onPlayerDurationChanged(long duration) {
        this.duration = duration;
        Platform.runLater(() -> durationText.setText(TimeUtils.format(duration)));
    }

    private void onPlayerTimeChanged(long time) {
        var progress = new AtomicReference<>(0d);

        if (duration != null && duration != 0) {
            progress.set((double) time / duration);
        }

        Platform.runLater(() -> {
            timeText.setText(TimeUtils.format(time));
            playbackProgress.setProgress(progress.get());
        });
    }

    private void onPlayerStateChanged(PlayerState state) {
        switch (state) {
            case PLAYING -> updatePlayState(true);
            case PAUSED -> updatePlayState(false);
            case LOADING -> Platform.runLater(() -> playbackProgress.setProgress(ProgressBar.INDETERMINATE_PROGRESS));
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
            progressPercentage.setText(ProgressUtils.progressToPercentage(status));
            downloadText.setText(ProgressUtils.progressToDownload(status));
            uploadText.setText(ProgressUtils.progressToUpload(status));
            activePeersText.setText(String.valueOf(status.seeds()));
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
