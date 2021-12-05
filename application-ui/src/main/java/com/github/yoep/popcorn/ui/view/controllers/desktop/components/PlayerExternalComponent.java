package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.yoep.player.adapter.Player;
import com.github.yoep.player.adapter.PlayerManagerService;
import com.github.yoep.player.adapter.listeners.PlayerListener;
import com.github.yoep.player.adapter.state.PlayerState;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.events.LoadUrlTorrentEvent;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.player.PlayerEventService;
import com.github.yoep.popcorn.ui.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import javafx.scene.input.MouseEvent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.event.EventListener;

import javax.annotation.PostConstruct;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@ViewController
@RequiredArgsConstructor
public class PlayerExternalComponent {
    private final ImageService imageService;
    private final PlayerEventService playerEventService;
    private final PlayerManagerService playerManagerService;

    private Long duration;

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

    //region Methods

    @EventListener
    public void onLoadMediaTorrent(LoadMediaTorrentEvent event) {
        reset();
        loadBackgroundImage(event.getMedia());
        Platform.runLater(() -> titleText.setText(event.getMedia().getTitle()));
    }

    @EventListener
    public void onLoadUrlTorrent(LoadUrlTorrentEvent event) {
        reset();
        Platform.runLater(() -> titleText.setText(event.getTorrentFileInfo().getFilename()));
    }

    //endregion

    //region Init

    @PostConstruct
    void Init() {
        playerEventService.addListener(createPlayerListener());
    }

    //region

    //region Functions

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
        Platform.runLater(() -> durationText.setText(formatTime(duration)));
    }

    private void onPlayerTimeChanged(long time) {
        var progress = new AtomicReference<>(0d);

        if (duration != null && duration != 0) {
            progress.set((double) time / duration);
        }

        Platform.runLater(() -> {
            timeText.setText(formatTime(time));
            playbackProgress.setProgress(progress.get());
        });
    }

    private void onPlayerStateChanged(PlayerState state) {
        switch (state) {
            case PLAYING:
                updatePlayState(true);
                break;
            case PAUSED:
                updatePlayState(false);
                break;
            case LOADING:
                Platform.runLater(() -> playbackProgress.setProgress(ProgressBar.INDETERMINATE_PROGRESS));
                break;
        }
    }

    private void onTogglePlaybackState() {
        playerManagerService.getActivePlayer()
                .ifPresent(e -> {
                    if (e.getState() == PlayerState.PAUSED) {
                        e.resume();
                    } else {
                        e.pause();
                    }
                });
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

    private String formatTime(long time) {
        return String.format("%02d:%02d",
                TimeUnit.MILLISECONDS.toMinutes(time),
                TimeUnit.MILLISECONDS.toSeconds(time) % 60);
    }

    private PlayerListener createPlayerListener() {
        return new PlayerListener() {
            @Override
            public void onDurationChanged(long newDuration) {
                onPlayerDurationChanged(newDuration);
            }

            @Override
            public void onTimeChanged(long newTime) {
                onPlayerTimeChanged(newTime);
            }

            @Override
            public void onStateChanged(PlayerState newState) {
                onPlayerStateChanged(newState);
            }
        };
    }

    @FXML
    void onPlayPauseClicked(MouseEvent event) {
        event.consume();
        onTogglePlaybackState();
    }

    @FXML
    void onStopClicked(MouseEvent event) {
        event.consume();
        playerManagerService.getActivePlayer()
                .ifPresent(Player::stop);
    }

    //endregion
}
