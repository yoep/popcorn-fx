package com.github.yoep.player.popcorn.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.yoep.player.popcorn.controls.ProgressSliderControl;
import com.github.yoep.player.popcorn.services.PlaybackService;
import com.github.yoep.popcorn.ui.view.services.FullscreenService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.input.MouseEvent;
import lombok.RequiredArgsConstructor;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.TimeUnit;

@ViewController
@RequiredArgsConstructor
public class PlayerControlsComponent implements Initializable {
    private final PlaybackService playbackService;
    private final FullscreenService fullscreenService;

    @FXML
    Icon playPauseIcon;
    @FXML
    Label timeLabel;
    @FXML
    ProgressSliderControl playProgress;
    @FXML
    Label durationLabel;
    @FXML
    Icon fullscreenIcon;

    //region Methods

    public void updatePlaybackState(boolean isPlaying) {
        if (isPlaying) {
            Platform.runLater(() -> playPauseIcon.setText(Icon.PAUSE_UNICODE));
        } else {
            Platform.runLater(() -> playPauseIcon.setText(Icon.PLAY_UNICODE));
        }
    }

    public void updateDuration(Long duration) {
        Platform.runLater(() -> {
            durationLabel.setText(formatTime(duration));
            playProgress.setDuration(duration);
        });
    }

    public void updateTime(Long time) {
        Platform.runLater(() -> {
            timeLabel.setText(formatTime(time));

            if (!playProgress.isValueChanging())
                playProgress.setTime(time);
        });
    }

    public void updateFullscreenState(Boolean isFullscreen) {
        if (isFullscreen) {
            Platform.runLater(() -> fullscreenIcon.setText(Icon.COMPRESS_UNICODE));
        } else {
            Platform.runLater(() -> fullscreenIcon.setText(Icon.EXPAND_UNICODE));
        }
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeSlider();
    }

    //endregion

    //region Functions

    private void initializeSlider() {
        playProgress.valueChangingProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue) {
                playbackService.pause();
            } else {
                playbackService.resume();
            }
        });
        playProgress.timeProperty().addListener((observableValue, oldValue, newValue) -> {
            if (playProgress.isValueChanging()) {
                playbackService.seek(newValue.longValue());
            }
        });

        playProgress.setOnMouseReleased(event -> setVideoTime(playProgress.getTime() + 1.0));
    }

    private String formatTime(long time) {
        return String.format("%02d:%02d",
                TimeUnit.MILLISECONDS.toMinutes(time),
                TimeUnit.MILLISECONDS.toSeconds(time) % 60);
    }

    private void setVideoTime(double time) {
        playProgress.setValueChanging(true);
        playProgress.setTime((long) time);
        playProgress.setValueChanging(false);
    }

    @FXML
    void onPlayPauseClicked(MouseEvent event) {
        event.consume();
        playbackService.togglePlayerPlaybackState();
    }

    @FXML
    void onFullscreenClicked(MouseEvent event) {
        event.consume();
        fullscreenService.toggle();
    }

    @FXML
    void onSubtitleSmaller() {
//        onSubtitleSizeChanged(-4);
    }

    @FXML
    void onSubtitleLarger() {
//        onSubtitleSizeChanged(4);
    }

    //endregion
}
