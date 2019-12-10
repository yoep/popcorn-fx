package com.github.yoep.popcorn.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.media.video.VideoPlayer;
import com.github.yoep.popcorn.media.video.state.PlayerState;
import com.github.yoep.popcorn.media.video.time.TimeListener;
import javafx.animation.PauseTransition;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.Slider;
import javafx.scene.layout.Pane;
import javafx.util.Duration;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Component;

import javax.annotation.PreDestroy;
import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.TimeUnit;

@Component
@RequiredArgsConstructor
public class PlayerComponent implements Initializable {
    private final PauseTransition idle = new PauseTransition(Duration.seconds(3));
    private final ActivityManager activityManager;

    private VideoPlayer videoPlayer;

    @FXML
    private Pane videoView;
    @FXML
    private Label title;
    @FXML
    private Label currentTime;
    @FXML
    private Label duration;
    @FXML
    private Slider slider;
    @FXML
    private Icon playPauseIcon;
    @FXML
    private Icon fullscreenIcon;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeVideoPlayer();
        initializeListeners();
        initializeSlider();
    }

    @PreDestroy
    private void dispose() {
        videoPlayer.dispose();
    }

    private void initializeVideoPlayer() {
        if (videoPlayer != null)
            return;

        this.videoPlayer = new VideoPlayer(videoView);
        this.videoPlayer.addListener((oldState, newState) -> {
            switch (newState) {
                case PLAYING:
                    Platform.runLater(() -> playPauseIcon.setText(Icon.PAUSE_UNICODE));
                    break;
                case PAUSED:
                    Platform.runLater(() -> playPauseIcon.setText(Icon.PLAY_UNICODE));
                    break;
            }
        });
        this.videoPlayer.addListener(new TimeListener() {
            @Override
            public void onTimeChanged(long newTime) {
                Platform.runLater(() -> {
                    currentTime.setText(formatTime(newTime));
                    slider.setValue(newTime);
                });
            }

            @Override
            public void onLengthChanged(long newLength) {
                Platform.runLater(() -> {
                    duration.setText(formatTime(newLength));
                    slider.setMax(newLength);
                });
            }
        });
    }

    private void initializeListeners() {
        activityManager.register(PlayMediaTrailerActivity.class, activity -> {
            videoPlayer.play(activity.getUrl());
            Platform.runLater(() -> title.setText(activity.getMedia().getTitle()));
        });
        activityManager.register(FullscreenActivity.class, activity -> {
            if (activity.isFullscreen()) {
                Platform.runLater(() -> fullscreenIcon.setText(Icon.COLLAPSE_UNICODE));
            } else {
                Platform.runLater(() -> fullscreenIcon.setText(Icon.EXPAND_UNICODE));
            }
        });
    }

    private void initializeSlider() {
        slider.valueProperty().addListener((observableValue, oldValue, newValue) -> {
            if (slider.isValueChanging()) {
                videoPlayer.setTime(newValue.longValue());
            }
        });
    }

    private void reset() {
        videoPlayer.stop();

        Platform.runLater(() -> {
            slider.setValue(0);
            currentTime.setText(formatTime(0));
            duration.setText(formatTime(0));
        });
    }

    private void changePlayPauseState() {
        if (videoPlayer.getPlayerState() == PlayerState.PAUSED) {
            videoPlayer.resume();
        } else {
            videoPlayer.pause();
        }
    }

    private String formatTime(long time) {
        return String.format("%02d:%02d",
                TimeUnit.MILLISECONDS.toMinutes(time),
                TimeUnit.MILLISECONDS.toSeconds(time) % 60);
    }

    @FXML
    private void onPlayerClick() {
        changePlayPauseState();
    }

    @FXML
    private void onPlayPauseClicked() {
        changePlayPauseState();
    }

    @FXML
    private void onFullscreenClicked() {
        activityManager.register(new ToggleFullscreenActivity() {
        });
    }

    @FXML
    private void close() {
        reset();
        activityManager.register(new PlayerCloseActivity() {
        });
    }
}
