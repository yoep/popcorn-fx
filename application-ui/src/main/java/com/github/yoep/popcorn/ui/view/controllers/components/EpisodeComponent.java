package com.github.yoep.popcorn.ui.view.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.view.controls.ImageCover;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.popcorn.ui.view.services.ShowHelperService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.function.Consumer;

@Slf4j
public class EpisodeComponent implements Initializable {
    static final String WATCHED_STYLE = "watched";

    private final Episode media;
    private final LocaleText localeText;
    private final ImageService imageService;

    private boolean watched;
    private Consumer<Boolean> onWatchClicked;
    private Runnable onDestroy;

    @FXML
    Pane graphic;
    @FXML
    ImageCover episodeArt;
    @FXML
    Icon watchedIcon;
    @FXML
    Label episodeNumber;
    @FXML
    Label title;
    @FXML
    Label airDate;
    @FXML
    Label synopsis;

    public EpisodeComponent(Episode media, LocaleText localeText, ImageService imageService) {
        this.media = media;
        this.localeText = localeText;
        this.imageService = imageService;
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        episodeArt.setImage(imageService.getArtHolder());
        episodeNumber.setText(String.valueOf(media.getEpisode()));
        title.setText(media.getTitle());
        airDate.setText(localeText.get(DetailsMessage.AIR_DATE, ShowHelperService.AIRED_DATE_PATTERN.format(media.getAirDate())));
        synopsis.setText(media.getSynopsis());
        graphic.parentProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue == null && onDestroy != null) {
                onDestroy.run();
            }
        });

        updateWatchedState(watched);
    }

    public void updateWatchedState(boolean newValue) {
        this.watched = newValue;

        if (graphic != null) {
            Platform.runLater(() -> {
                var styleClass = graphic.getStyleClass();

                if (newValue) {
                    styleClass.add(WATCHED_STYLE);
                } else {
                    styleClass.removeIf(e -> e.equals(WATCHED_STYLE));
                }

                watchedIcon.setText(newValue ? Icon.EYE_UNICODE : Icon.EYE_SLASH_UNICODE);
            });
        }
    }

    /**
     * Set the action which needs to be invoked when the watched icon is clicked.
     * It invoked the consumer with the new expected value for the watched state.
     */
    public void setOnWatchClicked(Consumer<Boolean> onWatchClicked) {
        this.onWatchClicked = onWatchClicked;
    }

    /**
     * Set the action to invoke when the episode component is being destroyed.
     */
    public void setOnDestroy(Runnable onDestroy) {
        this.onDestroy = onDestroy;
    }

    @FXML
    void onWatchedClicked(MouseEvent event) {
        event.consume();
        if (onWatchClicked != null) {
            onWatchClicked.accept(!watched);
        }
    }
}