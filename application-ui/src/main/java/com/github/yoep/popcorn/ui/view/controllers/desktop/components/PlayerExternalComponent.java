package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.utils.TimeUtils;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.events.LoadUrlTorrentEvent;
import com.github.yoep.popcorn.ui.player.PlayerEventService;
import com.github.yoep.popcorn.ui.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.popcorn.ui.view.services.PlayerExternalComponentService;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import javafx.scene.input.MouseEvent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.event.EventListener;

import javax.annotation.PostConstruct;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@ViewController
@RequiredArgsConstructor
public class PlayerExternalComponent {
    private final ImageService imageService;
    private final PlayerEventService playerEventService;
    private final PlatformProvider platformProvider;
    private final PlayerExternalComponentService playerExternalService;

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
        platformProvider.runOnRenderer(() -> titleText.setText(event.getMedia().getTitle()));
    }

    @EventListener
    public void onLoadUrlTorrent(LoadUrlTorrentEvent event) {
        reset();
        platformProvider.runOnRenderer(() -> titleText.setText(event.getTorrentFileInfo().getFilename()));
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
        platformProvider.runOnRenderer(() -> {
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
        platformProvider.runOnRenderer(() -> durationText.setText(TimeUtils.format(duration)));
    }

    private void onPlayerTimeChanged(long time) {
        var progress = new AtomicReference<>(0d);

        if (duration != null && duration != 0) {
            progress.set((double) time / duration);
        }

        platformProvider.runOnRenderer(() -> {
            timeText.setText(TimeUtils.format(time));
            playbackProgress.setProgress(progress.get());
        });
    }

    private void onPlayerStateChanged(PlayerState state) {
        switch (state) {
            case PLAYING -> updatePlayState(true);
            case PAUSED -> updatePlayState(false);
            case LOADING -> platformProvider.runOnRenderer(() -> playbackProgress.setProgress(ProgressBar.INDETERMINATE_PROGRESS));
        }
    }

    private void updatePlayState(boolean isPlaying) {
        platformProvider.runOnRenderer(() -> {
            if (isPlaying) {
                playPauseIcon.setText(Icon.PAUSE_UNICODE);
            } else {
                playPauseIcon.setText(Icon.PLAY_UNICODE);
            }
        });
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
        playerExternalService.togglePlaybackState();
    }

    @FXML
    void onStopClicked(MouseEvent event) {
        event.consume();
        playerExternalService.closePlayer();
    }

    //endregion
}
