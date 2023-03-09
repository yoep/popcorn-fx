package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayVideoEvent;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.events.WatchNowEvent;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.ui.utils.WatchNowUtils;
import com.github.yoep.popcorn.ui.view.controls.PlayerDropDownButton;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class DesktopMovieActionsComponent implements Initializable {
    private final PlayerManagerService playerService;
    private final EventPublisher eventPublisher;

    private MovieDetails media;

    @FXML
    PlayerDropDownButton watchNowButton;
    @FXML
    Button watchTrailerButton;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        WatchNowUtils.syncPlayerManagerAndWatchNowButton(playerService, watchNowButton);
        initializeListeners();
    }

    private void initializeListeners() {
        eventPublisher.register(ShowMovieDetailsEvent.class, event -> {
            this.media = event.getMedia();
            Platform.runLater(() -> {
                watchNowButton.select(playerService.getActivePlayer().orElse(null));
                watchTrailerButton.setVisible(StringUtils.isNotEmpty(media.getTrailer()));
                watchNowButton.requestFocus();
            });
            return event;
        });
    }

    private void onWatchNow() {
        eventPublisher.publish(new WatchNowEvent(this));
    }

    private void playTrailer() {
        eventPublisher.publish(new PlayVideoEvent(this, media.getTrailer(), media.getTitle(), false, media.getImages().getFanart()));
    }

    @FXML
    void onWatchNowClicked(MouseEvent event) {
        event.consume();
        onWatchNow();
    }

    @FXML
    void onWatchNowPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onWatchNow();
        }
    }

    @FXML
    void onTrailerClicked(MouseEvent event) {
        event.consume();
        playTrailer();
    }

    @FXML
    void onTrailerPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            playTrailer();
        }
    }
}
