package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Playlist;
import com.github.yoep.popcorn.backend.playlists.PlayNext;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.playlists.PlaylistManagerListener;
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
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class PlayingNextInComponent implements Initializable {
    private final ImageService imageService;
    private final PlaylistManager playlistManager;

    private Image posterPlaceholder = null;

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
        init();
    }

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeListeners();
        reset();
    }

    private void init() {
        imageService.getPosterPlaceholder().whenComplete((poster, throwable) -> {
            if (throwable == null) {
                posterPlaceholder = poster;
            } else {
                log.error("Failed to load poster placeholder, {}", throwable.getMessage(), throwable);
            }
        });
    }

    private void initializeListeners() {
        playlistManager.addListener(new PlaylistManagerListener() {
            @Override
            public void onPlaylistChanged() {
                reset();
            }

            @Override
            public void onPlayNextChanged(PlayNext next) {
                switch (next) {
                    case PlayNext.Next(var item) -> onItemChanged(item);
                    case PlayNext.End() -> onItemChanged(null);
                }
            }

            @Override
            public void onPlayNextIn(int seconds) {
                onPlayingIn(seconds);
            }

            @Override
            public void onPlayNextInAborted() {
                Platform.runLater(() -> playNextPane.setVisible(false));
            }

            @Override
            public void onStateChanged(Playlist.State state) {
                // no-op
            }
        });
    }

    //endregion

    //region Functions

    private void onPlayingIn(int playingIn) {
        onPlayingInChanged(playingIn);
    }

    private void onItemChanged(Playlist.Item nextItem) {
        reset();
        if (nextItem == null) {
            return;
        }

        Platform.runLater(() -> {
            showName.setText(nextItem.getTitle());
            episodeTitle.setText(nextItem.getCaption());
            episodeNumber.setText(Optional.ofNullable(nextItem.getMedia())
                    .filter(e -> nextItem.hasMedia())
                    .map(e -> {
                        if (e.hasEpisode()) {
                            return String.valueOf(e.getEpisode().getEpisode());
                        } else {
                            return null;
                        }
                    })
                    .orElse(null));
        });

        Optional.ofNullable(nextItem.getThumb()).ifPresentOrElse(
                e -> imageService.load(e).whenComplete((image, throwable) -> {
                    if (throwable == null) {
                        updatePoster(image);
                    } else {
                        updatePoster(posterPlaceholder);
                        log.error("Failed to load poster of next episode, {}", throwable.getMessage(), throwable);
                    }
                }),
                () -> onPosterLoaded(imageService.getPosterPlaceholder())
        );
    }

    private void onPlayingInChanged(int remainingTime) {
        Platform.runLater(() -> {
            playNextPane.setVisible(true);
            playingInCountdown.setText(String.valueOf(remainingTime));
            playNextPane.requestFocus();
        });
    }

    private void reset() {
        Platform.runLater(() -> {
            playNextPoster.setImage(posterPlaceholder);
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

    private void onPosterLoaded(CompletableFuture<Image> imageFuture) {
        imageFuture.whenComplete((image, throwable) -> {
            if (throwable == null) {
                updatePoster(image);
            } else {
                log.error("Failed to load poster of next episode, {}", throwable.getMessage(), throwable);
            }
        });
    }

    private synchronized void updatePoster(Image image) {
        Platform.runLater(() -> playNextPoster.setImage(image));
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
