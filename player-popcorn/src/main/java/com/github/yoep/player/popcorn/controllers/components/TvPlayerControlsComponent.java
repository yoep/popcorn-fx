package com.github.yoep.player.popcorn.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.player.popcorn.controls.ProgressControl;
import com.github.yoep.player.popcorn.listeners.PlayerControlsListener;
import com.github.yoep.player.popcorn.listeners.PlayerSubtitleListener;
import com.github.yoep.player.popcorn.services.PlayerControlsService;
import com.github.yoep.player.popcorn.services.PlayerSubtitleService;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.messages.SubtitleMessage;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.utils.TimeUtils;
import com.github.yoep.popcorn.ui.events.SubtitleOffsetEvent;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import javafx.application.Platform;
import javafx.event.Event;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class TvPlayerControlsComponent implements Initializable {
    static final int OFFSET_IN_SECONDS = 20;

    private final EventPublisher eventPublisher;
    private final PlayerControlsService playerControlsService;
    private final PlayerSubtitleService subtitleService;
    private final LocaleText localeText;

    @FXML
    Icon playButton;
    @FXML
    Label time;
    @FXML
    Label duration;
    @FXML
    ProgressControl timeline;
    @FXML
    Button subtitleIncreaseOffset;
    @FXML
    Button subtitleDecreaseOffset;
    @FXML
    Overlay subtitleOptionsOverlay;
    @FXML
    Overlay subtitleOverlay;
    @FXML
    AxisItemSelection<SubtitleInfo> subtitleSelection;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeListeners();
        initializeOverlay();
        initializeText();
    }

    private void initializeListeners() {
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

    private void initializeOverlay() {
        subtitleOverlay.shownProperty().addListener((observable, oldValue, newValue) -> {
            if (!newValue) {
                playerControlsService.resume();
            }
        });
        subtitleService.addListener(new PlayerSubtitleListener() {
            @Override
            public void onActiveSubtitleChanged(SubtitleInfo activeSubtitle) {
                Platform.runLater(() -> subtitleSelection.setSelectedItem(activeSubtitle));
            }

            @Override
            public void onAvailableSubtitlesChanged(List<SubtitleInfo> subtitles, SubtitleInfo activeSubtitle) {
                Platform.runLater(() -> {
                    subtitleSelection.setItems(subtitles.toArray(new SubtitleInfo[0]));
                    subtitleSelection.setSelectedItem(activeSubtitle);
                });
            }
        });
        subtitleSelection.setItemFactory(item -> new Button(item.getLanguage().getNativeName()));
        subtitleSelection.setOnItemActivated(item -> {
            subtitleOverlay.hide();
            playerControlsService.resume();
            subtitleService.updateActiveSubtitle(item);
        });
        subtitleSelection.setItems(subtitleService.defaultSubtitles());
    }

    private void initializeText() {
        subtitleIncreaseOffset.setText(localeText.get(SubtitleMessage.INCREASE_SUBTITLE_OFFSET, OFFSET_IN_SECONDS));
        subtitleDecreaseOffset.setText(localeText.get(SubtitleMessage.DECREASE_SUBTITLE_OFFSET, OFFSET_IN_SECONDS));
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

    private void onSubtitleSizeChanged(int pixelChange) {
        subtitleService.updateSubtitleSizeWithSizeOffset(pixelChange);
    }

    private void onSubtitleOffsetChanged(int offsetInSeconds) {
        eventPublisher.publish(new SubtitleOffsetEvent(this, offsetInSeconds));
    }

    private void showSubtitlesOverlay() {
        playerControlsService.pause();
        subtitleOptionsOverlay.hide();
        subtitleOverlay.show();
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

    @FXML
    void onSubtitleChange(Event event) {
        if (event instanceof KeyEvent keyEvent && keyEvent.getCode() != KeyCode.ENTER)
            return;

        event.consume();
        showSubtitlesOverlay();
    }

    @FXML
    void onIncreaseOffset(Event event) {
        if (event instanceof KeyEvent keyEvent && keyEvent.getCode() != KeyCode.ENTER)
            return;

        event.consume();
        onSubtitleOffsetChanged(OFFSET_IN_SECONDS);
    }

    @FXML
    void onDecreaseOffset(Event event) {
        if (event instanceof KeyEvent keyEvent && keyEvent.getCode() != KeyCode.ENTER)
            return;

        event.consume();
        onSubtitleOffsetChanged(-OFFSET_IN_SECONDS);
    }

    @FXML
    void onIncreaseFontSize(Event event) {
        if (event instanceof KeyEvent keyEvent && keyEvent.getCode() != KeyCode.ENTER)
            return;

        event.consume();
        onSubtitleSizeChanged(4);
    }

    @FXML
    void onDecreaseFontSize(Event event) {
        if (event instanceof KeyEvent keyEvent && keyEvent.getCode() != KeyCode.ENTER)
            return;

        event.consume();
        onSubtitleSizeChanged(-4);
    }

    @FXML
    void onChangeSubtitle(Event event) {
        event.consume();
        playerControlsService.pause();
        subtitleOverlay.show();
    }
}
