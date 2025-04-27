package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowSerieDetailsEvent;
import com.github.yoep.popcorn.backend.media.ShowDetails;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.font.controls.Icon;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.view.listeners.DetailsComponentListener;
import com.github.yoep.popcorn.ui.view.services.DetailsComponentService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Objects;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class TvSerieActionsComponent implements Initializable {
    private final EventPublisher eventPublisher;
    private final LocaleText localeText;
    private final DetailsComponentService detailsComponentService;

    private ShowDetails media;

    @FXML
    Button favoriteButton;
    @FXML
    Icon favoriteIcon;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        eventPublisher.register(ShowSerieDetailsEvent.class, event -> {
            this.media = event.getMedia();
            Platform.runLater(this::updateFavoriteState);
            return event;
        });
        detailsComponentService.addListener(new DetailsComponentListener() {
            @Override
            public void onWatchChanged(String imdbId, boolean newState) {
                // no-op
            }

            @Override
            public void onLikedChanged(String imdbId, boolean newState) {
                if (Objects.equals(media.id(), imdbId)) {
                    Platform.runLater(() -> updateFavoriteState());
                }
            }
        });
    }

    private void updateFavoriteState() {
        detailsComponentService.isLiked(media).whenComplete((state, throwable) -> {
            if (throwable == null) {
                Platform.runLater(() -> {
                    favoriteButton.setText(localeText.get(state ? DetailsMessage.REMOVE : DetailsMessage.ADD));
                    favoriteIcon.setText(state ? Icon.HEART_UNICODE : Icon.HEART_O_UNICODE);
                });
            } else {
                log.error("Failed to retrieve liked state", throwable);
            }
        });
    }

    private void toggleFavoriteState() {
        detailsComponentService.toggleLikedState(media);
    }

    @FXML
    void onFavoriteClicked(MouseEvent event) {
        event.consume();
        toggleFavoriteState();
    }

    @FXML
    void onFavoritePressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            toggleFavoriteState();
        }
    }
}
