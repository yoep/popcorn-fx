package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.ui.events.PlayVideoEvent;
import com.github.yoep.popcorn.ui.view.controllers.common.components.AbstractPlayerControlsComponent;
import com.github.yoep.popcorn.ui.view.controls.ProgressControl;
import com.github.yoep.popcorn.ui.view.services.VideoPlayerService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.event.EventListener;

@Slf4j
public class PlayerControlsComponent extends AbstractPlayerControlsComponent {
    @FXML
    private ProgressControl progress;

    //region Constructors

    public PlayerControlsComponent(VideoPlayerService videoPlayerService) {
        super(videoPlayerService);
    }

    //endregion

    //region Methods

    @EventListener(PlayVideoEvent.class)
    public void onPlayVideo() {
        Platform.runLater(() -> playPauseIcon.requestFocus());
    }

    //endregion

    //region AbstractPlayerControlsComponent

    @Override
    protected void onTimeChanged(Number newValue) {
        super.onTimeChanged(newValue);
        progress.setTime(newValue.longValue());
    }

    @Override
    protected void onDurationChanged(Number newValue) {
        super.onDurationChanged(newValue);
        progress.setDuration(newValue.longValue());
    }

    @Override
    protected void onProgressChanged(double newValue) {
        progress.setLoadProgress(newValue);
    }

    @Override
    protected void reset() {
        super.reset();
        progress.reset();
    }

    //endregion

    //region Functions

    private void onPlayPause() {
        videoPlayerService.changePlayPauseState();
    }

    private void onBackward() {
        videoPlayerService.videoTimeOffset(-5000);
    }

    private void onForward() {
        videoPlayerService.videoTimeOffset(5000);
    }

    private void close() {
        videoPlayerService.close();
    }

    @FXML
    private void onCloseClicked(MouseEvent event) {
        event.consume();
        close();
    }

    @FXML
    private void onCloseKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            close();
        }
    }

    @FXML
    private void onBackwardClicked(MouseEvent event) {
        event.consume();
        onBackward();
    }

    @FXML
    private void onBackwardKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onBackward();
        }
    }

    @FXML
    private void onPlayPauseClicked(MouseEvent event) {
        event.consume();
        onPlayPause();
    }

    @FXML
    private void onPlayPauseKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onPlayPause();
        }
    }

    @FXML
    private void onForwardClicked(MouseEvent event) {
        event.consume();
        onForward();
    }

    @FXML
    private void onForwardKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onForward();
        }
    }

    //endregion
}
