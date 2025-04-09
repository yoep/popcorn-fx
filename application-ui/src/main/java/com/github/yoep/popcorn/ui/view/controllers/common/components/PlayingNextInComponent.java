package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Playlist;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.playlists.PlaylistManagerListener;
import com.github.yoep.popcorn.backend.playlists.PlaylistState;
import com.github.yoep.popcorn.ui.view.controls.SizedImageView;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.image.Image;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Objects;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class PlayingNextInComponent implements Initializable {
    private final ImageService imageService;
    private final PlaylistManager playlistManager;

    private Long lastKnownPlayingIn;
    private Playlist.Item lastKnownItem;

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

    public PlayingNextInComponent(ImageService imageService, PlaylistManager playlistManager) {
        this.imageService = imageService;
        this.playlistManager = playlistManager;
    }

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
            public void onPlayingIn(Long playingIn, Playlist.Item item) {
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

    private void onPlayingIn(Long playingIn, Playlist.Item item) {
        if (!Objects.equals(lastKnownItem, item)) {
            onItemChanged(item);
        }

        onPlayingInChanged(playingIn);
        this.lastKnownPlayingIn = playingIn;
        this.lastKnownItem = item;
    }

    private void onItemChanged(Playlist.Item nextItem) {
        reset();

        if (nextItem == null) {
            return;
        }

        Platform.runLater(() -> {
            showName.setText(nextItem.getTitle());
            episodeTitle.setText(nextItem.getCaption());
            // TODO
            //episodeNumber.setText(String.valueOf(nextItem.getEpisode().getEpisode()));
        });

        Optional.ofNullable(nextItem.getThumb()).ifPresentOrElse(
                e -> imageService.load(e).whenComplete((image, throwable) -> {
                    if (throwable == null) {
                        playNextPoster.setImage(image);
                    } else {
                        updatePoster(imageService.getPosterPlaceholder());
                        log.error("Failed to load poster of next episode, {}", throwable.getMessage(), throwable);
                    }
                }),
                () -> updatePoster(imageService.getPosterPlaceholder())
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
        updatePoster(imageService.getPosterPlaceholder());

        Platform.runLater(() -> {
            playNextPane.setVisible(false);
            showName.setText(null);
            episodeTitle.setText(null);
            episodeNumber.setText(null);
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

    private void updatePoster(CompletableFuture<Image> imageFuture) {
        imageFuture.whenComplete((image, throwable) -> {
            if (throwable == null) {
                Platform.runLater(() -> playNextPoster.setImage(image));
            } else {
                log.error("Failed to load poster of next episode, {}", throwable.getMessage(), throwable);
            }
        });
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
