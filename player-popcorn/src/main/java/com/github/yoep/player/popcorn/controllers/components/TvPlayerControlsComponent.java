package com.github.yoep.player.popcorn.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.player.popcorn.controls.ProgressControl;
import com.github.yoep.player.popcorn.listeners.PlayerControlsListener;
import com.github.yoep.player.popcorn.services.PlayerControlsService;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.utils.TimeUtils;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class TvPlayerControlsComponent implements Initializable {
    private final EventPublisher eventPublisher;
    private final PlayerControlsService playerControlsService;

    @FXML
    Icon playButton;
    @FXML
    Label time;
    @FXML
    Label duration;
    @FXML
    ProgressControl timeline;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        playButton.sceneProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue != null) {
                Platform.runLater(() -> playButton.requestFocus());
            }
        });
        playerControlsService.addListener(new PlayerControlsListener() {
            @Override
            public void onFullscreenStateChanged(boolean isFullscreenEnabled) {

            }

            @Override
            public void onSubtitleStateChanged(boolean isSubtitlesEnabled) {
                // no-op
            }

            @Override
            public void onPlayerStateChanged(PlayerState state) {
                Platform.runLater(() -> {
                    switch (state) {
                        case PLAYING -> playButton.setText(Icon.PAUSE_UNICODE);
                        case PAUSED -> playButton.setText(Icon.PLAY_UNICODE);
                        case BUFFERING -> playButton.setText(Icon.SPINNER_UNICODE);
                        case ERROR -> playButton.setText(Icon.BAN_UNICODE);
                    }
                });
            }

            @Override
            public void onPlayerTimeChanged(long time) {
                Platform.runLater(() -> {
                    TvPlayerControlsComponent.this.time.setText(TimeUtils.format(time));
                    timeline.setTime(time);
                });
            }

            @Override
            public void onPlayerDurationChanged(long duration) {
                Platform.runLater(() -> {
                    TvPlayerControlsComponent.this.duration.setText(TimeUtils.format(duration));
                    timeline.setDuration(duration);
                });
            }

            @Override
            public void onDownloadStatusChanged(DownloadStatus progress) {
                Platform.runLater(() -> timeline.setLoadProgress(progress.getProgress()));
            }

            @Override
            public void onVolumeChanged(int volume) {
                // no-op
            }
        });
    }

    private void closePlayer() {
        eventPublisher.publish(new ClosePlayerEvent(this, ClosePlayerEvent.Reason.USER));
    }

    private void reverse() {
        playerControlsService.seek(playerControlsService.getTime() - 10000);
    }

    private void forward() {
        playerControlsService.seek(playerControlsService.getTime() + 10000);
    }

    @FXML
    void onPlayClicked(MouseEvent event) {
        event.consume();
        playerControlsService.togglePlayerPlaybackState();
    }

    @FXML
    void onPlayPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            playerControlsService.togglePlayerPlaybackState();
        }
    }

    @FXML
    void onStopClicked(MouseEvent event) {
        event.consume();
        closePlayer();
    }

    @FXML
    void onStopPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            closePlayer();
        }
    }

    @FXML
    void onReverseClicked(MouseEvent event) {
        event.consume();
        reverse();
    }

    @FXML
    void onReversePressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            reverse();
        }
    }

    @FXML
    void onForwardClicked(MouseEvent event) {
        event.consume();
        forward();
    }

    @FXML
    void onForwardPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            forward();
        }
    }
}
