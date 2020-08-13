package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.ui.media.providers.models.Episode;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.popcorn.ui.view.services.PlayNextService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.image.ImageView;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class PlayerPlayNextComponent implements Initializable {
    private final ImageService imageService;
    private final PlayNextService playNextService;

    @FXML
    private Pane playNextPane;
    @FXML
    private ImageView playNextPoster;
    @FXML
    private Label showName;
    @FXML
    private Label episodeTitle;
    @FXML
    private Label episodeNumber;
    @FXML
    private Label playingInCountdown;

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        reset();
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializeListeners();
    }

    private void initializeListeners() {
        playNextService.nextEpisodeProperty().addListener((observable, oldValue, newValue) -> onEpisodeChanged(newValue));
        playNextService.playingInProperty().addListener((observable, oldValue, newValue) -> onPlayingInChanged(newValue.longValue()));
    }

    //endregion

    //region Functions

    private void onEpisodeChanged(Episode episode) {
        reset();

        if (episode == null) {
            return;
        }

        var show = episode.getShow();

        Platform.runLater(() -> {
            showName.setText(show.getTitle());
            episodeTitle.setText(episode.getTitle());
            episodeNumber.setText(String.valueOf(episode.getEpisode()));
        });

        imageService.loadPoster(episode, 100, 140).whenComplete((image, throwable) -> {
            if (throwable == null) {
                image.ifPresentOrElse(playNextPoster::setImage,
                        () -> playNextPoster.setImage(imageService.getPosterHolder()));
            } else {
                playNextPoster.setImage(imageService.getPosterHolder());
                log.error("Failed to load poster of next episode, " + throwable.getMessage(), throwable);
            }
        });
    }

    private void onPlayingInChanged(long remainingTime) {
        Platform.runLater(() -> {
            playNextPane.setVisible(true);
            playingInCountdown.setText(String.valueOf(remainingTime));
        });
    }

    private void reset() {
        Platform.runLater(() -> {
            playNextPane.setVisible(false);
            showName.setText(null);
            episodeTitle.setText(null);
            episodeNumber.setText(null);
            playNextPoster.setImage(imageService.getPosterHolder());
        });
    }

    @FXML
    private void onPlayNextClicked(MouseEvent event) {
        event.consume();
        playNextService.playNextEpisodeNow();
        reset();
    }

    //endregion
}
