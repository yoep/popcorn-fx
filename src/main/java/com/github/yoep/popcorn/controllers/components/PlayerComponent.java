package com.github.yoep.popcorn.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.media.video.VideoPlayer;
import com.github.yoep.popcorn.media.video.state.PlayerState;
import com.github.yoep.popcorn.media.video.time.TimeListener;
import com.github.yoep.popcorn.services.TorrentService;
import javafx.animation.PauseTransition;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.Slider;
import javafx.scene.layout.Pane;
import javafx.util.Duration;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;
import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.TimeUnit;

@Slf4j
@Component
@RequiredArgsConstructor
public class PlayerComponent implements Initializable {
    private final PauseTransition idle = new PauseTransition(Duration.seconds(3));
    private final ActivityManager activityManager;
    private final TaskExecutor taskExecutor;
    private final TorrentService torrentService;

    private VideoPlayer videoPlayer;

    @FXML
    private Pane playerPane;
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
        initializeKeyEvents();
        initializeVideoPlayer();
        initializeSlider();
    }

    @PostConstruct
    private void init() {
        initializeListeners();
    }

    @PreDestroy
    private void dispose() {
        videoPlayer.dispose();
    }

    private void initializeKeyEvents() {
        playerPane.setOnKeyReleased(event -> {
            switch (event.getCode()) {
                case LEFT:
                case KP_LEFT:
                    increaseVideoTime(-5000);
                    break;
                case RIGHT:
                case KP_RIGHT:
                    increaseVideoTime(5000);
                    break;
                case SPACE:
                    changePlayPauseState();
                    break;
                case F11:
                    toggleFullscreen();
                    break;
            }
        });
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
                case FINISHED:
                    //TODO: fix issue were this is being called when the video is switched
                    //close();
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
        activityManager.register(PlayVideoActivity.class, activity -> {
            videoPlayer.play(activity.getUrl());
            Platform.runLater(() -> title.setText(activity.getMedia().getTitle()));
        });
        activityManager.register(LoadMovieActivity.class, activity -> {

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
        slider.valueChangingProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue) {
                videoPlayer.pause();
            } else {
                videoPlayer.resume();
            }
        });
        slider.valueProperty().addListener((observableValue, oldValue, newValue) -> {
            if (slider.isValueChanging()) {
                videoPlayer.setTime(newValue.longValue());
            }
        });
        slider.setOnMouseReleased(event -> setVideoTime(slider.getValue() + 1));
    }

    private void reset() {
        taskExecutor.execute(() -> videoPlayer.stop());

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

    private void toggleFullscreen() {
        log.trace("Toggling full screen mode");
        activityManager.register(new ToggleFullscreenActivity() {
        });
    }

    private void increaseVideoTime(double amount) {
        log.trace("Increasing video time with {}", amount);
        double newSliderValue = slider.getValue() + amount;
        double maxSliderValue = slider.getMax();

        if (newSliderValue > maxSliderValue)
            newSliderValue = maxSliderValue;

        setVideoTime(newSliderValue);
    }

    private void setVideoTime(double time) {
        slider.setValueChanging(true);
        slider.setValue(time);
        slider.setValueChanging(false);
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
        toggleFullscreen();
    }

    @FXML
    private void close() {
        log.trace("Closing player component");
        reset();
        torrentService.stopStream();
        activityManager.register(new PlayerCloseActivity() {
        });
    }
}
