package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.playlists.DefaultPlaylistManager;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.playlists.model.PlaylistItem;
import com.github.yoep.popcorn.backend.playlists.PlaylistManagerListener;
import com.github.yoep.popcorn.backend.playlists.PlaylistState;
import com.github.yoep.popcorn.ui.view.controls.SizedImageView;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Objects;
import java.util.Optional;
import java.util.ResourceBundle;

@Slf4j

@RequiredArgsConstructor
public class PlayingNextInComponent implements Initializable {
    private final ImageService imageService;
    private final PlaylistManager playlistManager;

    private Long lastKnownPlayingIn;
    private PlaylistItem lastKnownItem;

    @FXML
    Pane playNextPane;
    @FXML
    SizedImageView playNextPoster;
    @FXML
    Label showName;
    @FXML
    Label episodeTitle;
    @FXML
    Label episodeNumber;
    @FXML
    Label playingInCountdown;

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeListeners();
        reset();
    }

    private void initializeListeners() {
        playlistManager.addListener(new PlaylistManagerListener() {
            @Override
            public void onPlaylistChanged() {
                // no-op
            }

            @Override
            public void onPlayingIn(Long playingIn, PlaylistItem item) {
                PlayingNextInComponent.this.onPlayingIn(playingIn, item);
            }

            @Override
            public void onStateChanged(PlaylistState state) {
                // no-op
            }
        });
    }

    //endregion

    //region Functions

    private void onPlayingIn(Long playingIn, PlaylistItem item) {
        if (!Objects.equals(lastKnownItem, item)) {
            onItemChanged(item);
        }

        onPlayingInChanged(playingIn);
        this.lastKnownPlayingIn = playingIn;
        this.lastKnownItem = item;
    }

    private void onItemChanged(PlaylistItem nextItem) {
        reset();

        if (nextItem == null) {
            return;
        }

        Platform.runLater(() -> {
            showName.setText(nextItem.title());
            episodeTitle.setText(nextItem.getCaption().orElse(null));
            //TODO
            //            episodeNumber.setText(String.valueOf(nextItem.getEpisode().getEpisode()));
        });

        nextItem.getThumb().ifPresentOrElse(
                e -> imageService.load(e).whenComplete((image, throwable) -> {
                    if (throwable == null) {
                        playNextPoster.setImage(image);
                    } else {
                        playNextPoster.setImage(imageService.getPosterPlaceholder());
                        log.error("Failed to load poster of next episode, " + throwable.getMessage(), throwable);
                    }
                }),
                () -> playNextPoster.setImage(imageService.getPosterPlaceholder())
        );
    }

    private void onPlayingInChanged(Long remainingTime) {
        Platform.runLater(() -> {
            playNextPane.setVisible(remainingTime != null);
            playingInCountdown.setText(Optional.ofNullable(remainingTime)
                    .map(String::valueOf)
                    .orElse(null));
            focusPlayingNextIfNeeded(remainingTime);
        });
    }

    private void focusPlayingNextIfNeeded(Long remainingTime) {
        if (lastKnownPlayingIn == null && remainingTime != null) {
            playNextPane.requestFocus();
        }
    }

    private void reset() {
        this.lastKnownPlayingIn = null;

        Platform.runLater(() -> {
            playNextPane.setVisible(false);
            showName.setText(null);
            episodeTitle.setText(null);
            episodeNumber.setText(null);
            playNextPoster.setImage(imageService.getPosterPlaceholder());
        });
    }

    private void onPlayNextNow() {
        playlistManager.playNext();
        reset();
    }

    private void onPlayNextStop() {
        playlistManager.stop();
        reset();
    }

    @FXML
    void onPlayNextClicked(MouseEvent event) {
        event.consume();
        onPlayNextNow();
    }

    @FXML
    void onPlayNextPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onPlayNextNow();
        }
    }

    @FXML
    void onPlayNextStopClicked(MouseEvent event) {
        event.consume();
        onPlayNextStop();
    }

    @FXML
    void onPlayNextStopPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onPlayNextStop();
        }
    }

    //endregion
}
