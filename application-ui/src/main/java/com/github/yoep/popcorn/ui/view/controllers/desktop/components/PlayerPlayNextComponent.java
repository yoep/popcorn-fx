package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.ui.playnext.PlayNextService;
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
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class PlayerPlayNextComponent implements Initializable {
    private final ImageService imageService;
    private final PlayNextService playNextService;

    private long lastKnownPlayingIn;

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
        playNextService.nextEpisodeProperty().addListener((observable, oldValue, newValue) -> onEpisodeChanged(newValue));
        playNextService.playingInProperty().addListener((observable, oldValue, newValue) -> onPlayingInChanged(newValue.longValue()));
    }

    //endregion

    //region Functions

    private void onEpisodeChanged(PlayNextService.NextEpisode nextEpisode) {
        reset();

        if (nextEpisode == null) {
            return;
        }

        var show = nextEpisode.getShow();

        Platform.runLater(() -> {
            showName.setText(show.getTitle());
            episodeTitle.setText(nextEpisode.getEpisode().getTitle());
            episodeNumber.setText(String.valueOf(nextEpisode.getEpisode().getEpisode()));
        });

        imageService.loadPoster(nextEpisode.getShow()).whenComplete((image, throwable) -> {
            if (throwable == null) {
                image.ifPresentOrElse(playNextPoster::setImage,
                        () -> playNextPoster.setImage(imageService.getPosterPlaceholder()));
            } else {
                playNextPoster.setImage(imageService.getPosterPlaceholder());
                log.error("Failed to load poster of next episode, " + throwable.getMessage(), throwable);
            }
        });
    }

    private void onPlayingInChanged(long remainingTime) {
        Platform.runLater(() -> {
            playNextPane.setVisible(remainingTime != PlayNextService.UNDEFINED);
            playingInCountdown.setText(String.valueOf(remainingTime));
            focusPlayingNextIfNeeded(remainingTime);

            lastKnownPlayingIn = remainingTime;
        });
    }

    private void focusPlayingNextIfNeeded(long remainingTime) {
        if (lastKnownPlayingIn == PlayNextService.UNDEFINED && remainingTime != PlayNextService.UNDEFINED) {
            playNextPane.requestFocus();
        }
    }

    private void reset() {
        Platform.runLater(() -> {
            playNextPane.setVisible(false);
            showName.setText(null);
            episodeTitle.setText(null);
            episodeNumber.setText(null);
            playNextPoster.setImage(imageService.getPosterPlaceholder());
        });
    }

    private void onPlayNextNow() {
        playNextService.playNextEpisodeNow();
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
        playNextService.stop();
        reset();
    }

    //endregion
}
