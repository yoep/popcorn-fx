package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.ui.activities.ActivityManager;
import com.github.yoep.popcorn.ui.activities.ClosePlayerActivity;
import com.github.yoep.popcorn.ui.view.services.VideoPlayerService;
import com.github.yoep.video.adapter.state.PlayerState;
import javafx.application.Platform;
import javafx.beans.value.ChangeListener;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;
import java.util.concurrent.TimeUnit;

@Slf4j
public abstract class AbstractPlayerControlsComponent {
    protected final ActivityManager activityManager;
    protected final VideoPlayerService videoPlayerService;

    private final ChangeListener<PlayerState> playerStateListener = (observable, oldValue, newValue) -> onPlayerStateChanged(newValue);
    private final ChangeListener<Number> timeListener = (observable, oldValue, newValue) -> onTimeChanged(newValue);
    private final ChangeListener<Number> durationListener = (observable, oldValue, newValue) -> onDurationChanged(newValue);

    @FXML
    protected Icon playPauseIcon;
    @FXML
    protected Label timeLabel;
    @FXML
    protected Label durationLabel;

    //region Constructors

    public AbstractPlayerControlsComponent(ActivityManager activityManager, VideoPlayerService videoPlayerService) {
        this.activityManager = activityManager;
        this.videoPlayerService = videoPlayerService;
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializeActivityListeners();
        initializeVideoListeners();
    }

    protected void initializeActivityListeners() {
        activityManager.register(ClosePlayerActivity.class, this::onClose);
    }

    protected void initializeVideoListeners() {
        videoPlayerService.videoPlayerProperty().addListener((observable, oldValue, newValue) -> {
            if (oldValue != null) {
                oldValue.playerStateProperty().removeListener(playerStateListener);
                oldValue.timeProperty().removeListener(timeListener);
                oldValue.durationProperty().removeListener(durationListener);
            }

            newValue.playerStateProperty().addListener(playerStateListener);
            newValue.timeProperty().addListener(timeListener);
            newValue.durationProperty().addListener(durationListener);
        });
    }

    //endregion

    //region Functions

    /**
     * Reset this component to it's idle state.
     */
    protected void reset() {
        Platform.runLater(() -> {
            timeLabel.setText(formatTime(0));
            durationLabel.setText(formatTime(0));
        });
    }

    protected String formatTime(long time) {
        return String.format("%02d:%02d",
                TimeUnit.MILLISECONDS.toMinutes(time),
                TimeUnit.MILLISECONDS.toSeconds(time) % 60);
    }

    private void onPlayerStateChanged(PlayerState newValue) {
        switch (newValue) {
            case PLAYING:
                Platform.runLater(() -> playPauseIcon.setText(Icon.PAUSE_UNICODE));
                break;
            case PAUSED:
                Platform.runLater(() -> playPauseIcon.setText(Icon.PLAY_UNICODE));
                break;
        }
    }

    protected void onTimeChanged(Number newValue) {
        Platform.runLater(() -> {
            long time = newValue.longValue();

            timeLabel.setText(formatTime(time));
        });
    }

    protected void onDurationChanged(Number newValue) {
        Platform.runLater(() -> {
            long duration = newValue.longValue();

            durationLabel.setText(formatTime(duration));
        });
    }

    private void onClose(ClosePlayerActivity activity) {
        reset();
    }

    //endregion
}
