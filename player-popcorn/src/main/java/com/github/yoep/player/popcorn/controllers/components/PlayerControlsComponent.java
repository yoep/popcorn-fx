package com.github.yoep.player.popcorn.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.yoep.player.popcorn.controls.ProgressSliderControl;
import com.github.yoep.player.popcorn.controls.Volume;
import com.github.yoep.player.popcorn.listeners.PlayerControlsListener;
import com.github.yoep.player.popcorn.services.PlayerControlsService;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.events.PlayerStoppedEvent;
import com.github.yoep.popcorn.backend.utils.TimeUtils;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.event.EventListener;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@ViewController
@RequiredArgsConstructor
public class PlayerControlsComponent implements Initializable {
    private final PlayerControlsService playerControlsService;
    private final PlatformProvider platformProvider;

    @FXML
    Icon playPauseIcon;
    @FXML
    Label timeLabel;
    @FXML
    ProgressSliderControl playProgress;
    @FXML
    Label durationLabel;
    @FXML
    Volume volumeIcon;
    @FXML
    Icon fullscreenIcon;
    @FXML
    Pane playerActions;
    @FXML
    Pane subtitleSection;

    //region Methods

    private void onFullscreenStateChanged(Boolean isFullscreen) {
        if (isFullscreen) {
            platformProvider.runOnRenderer(() -> fullscreenIcon.setText(Icon.COMPRESS_UNICODE));
        } else {
            platformProvider.runOnRenderer(() -> fullscreenIcon.setText(Icon.EXPAND_UNICODE));
        }
    }

    @EventListener(PlayerStoppedEvent.class)
    public void reset() {
        platformProvider.runOnRenderer(() -> {
            playProgress.setTime(0);
            playerActions.getChildren().remove(subtitleSection);
        });
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeSlider();
        initializeListeners();
    }

    private void initializeSlider() {
        playProgress.valueChangingProperty().addListener((observable, oldValue, newValue) ->
                playerControlsService.onSeekChanging(newValue));
        playProgress.timeProperty().addListener((observableValue, oldValue, newValue) ->
                onSeeking(newValue));
        volumeIcon.volumeProperty().addListener((observable, oldValue, newValue) -> playerControlsService.onVolumeChanged(newValue.doubleValue()));

        playProgress.setOnMouseReleased(event -> setVideoTime(playProgress.getTime() + 1.0));
    }

    private void initializeListeners() {
        playerControlsService.addListener(new PlayerControlsListener() {
            @Override
            public void onFullscreenStateChanged(boolean isFullscreenEnabled) {
                PlayerControlsComponent.this.onFullscreenStateChanged(isFullscreenEnabled);
            }

            @Override
            public void onSubtitleStateChanged(boolean isSubtitlesEnabled) {
                onSubtitleVisibilityChanged(isSubtitlesEnabled);
            }

            @Override
            public void onPlayerStateChanged(PlayerState state) {
                PlayerControlsComponent.this.onPlayerStateChanged(state == PlayerState.PLAYING);
            }

            @Override
            public void onPlayerTimeChanged(long time) {
                onTimeChanged(time);
            }

            @Override
            public void onPlayerDurationChanged(long duration) {
                onDurationChanged(duration);
            }

            @Override
            public void onDownloadStatusChanged(DownloadStatus progress) {
                PlayerControlsComponent.this.onDownloadStatusChanged(progress);
            }

            @Override
            public void onVolumeChanged(int volume) {
                PlayerControlsComponent.this.onVolumeChanged(volume);
            }
        });

        playerControlsService.retrieveValues();
    }

    //endregion

    //region Functions

    private void onPlayerStateChanged(boolean isPlaying) {
        if (isPlaying) {
            platformProvider.runOnRenderer(() -> playPauseIcon.setText(Icon.PAUSE_UNICODE));
        } else {
            platformProvider.runOnRenderer(() -> playPauseIcon.setText(Icon.PLAY_UNICODE));
        }
    }

    private void onDurationChanged(Long duration) {
        platformProvider.runOnRenderer(() -> {
            durationLabel.setText(TimeUtils.format(duration));
            playProgress.setDuration(duration);
        });
    }

    private void onTimeChanged(Long time) {
        platformProvider.runOnRenderer(() -> {
            timeLabel.setText(TimeUtils.format(time));

            if (!playProgress.isValueChanging())
                playProgress.setTime(time);
        });
    }

    private void onSubtitleVisibilityChanged(boolean isVisible) {
        // update the visibility of the subtitles section
        platformProvider.runOnRenderer(() -> {
            if (isVisible && !playerActions.getChildren().contains(subtitleSection)) {
                playerActions.getChildren().add(0, subtitleSection);
            } else {
                playerActions.getChildren().remove(subtitleSection);
            }
        });
    }

    private void onSeeking(Number newValue) {
        // check if the play progress is seeking a timestamp
        // if not, ignore this invocation
        if (!playProgress.isValueChanging()) {
            return;
        }

        playerControlsService.seek(newValue.longValue());
        timeLabel.setText(TimeUtils.format(newValue.longValue()));
    }

    private void onDownloadStatusChanged(DownloadStatus progress) {
        playProgress.setLoadProgress(progress.getProgress());
    }

    private void onVolumeChanged(int volume) {
        if (!volumeIcon.isValueChanging()) {
            volumeIcon.setVolume((double) volume / 100);
        }
    }

    private void setVideoTime(double time) {
        playProgress.setValueChanging(true);
        playProgress.setTime((long) time);
        playProgress.setValueChanging(false);
    }

    @FXML
    void onPlayPauseClicked(MouseEvent event) {
        event.consume();
        playerControlsService.togglePlayerPlaybackState();
    }

    @FXML
    void onFullscreenClicked(MouseEvent event) {
        event.consume();
        playerControlsService.toggleFullscreen();
    }

    //endregion
}
