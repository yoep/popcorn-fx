package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.media.Episode;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.font.controls.Icon;
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
import lombok.Setter;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.function.Consumer;

@Slf4j
public class EpisodeComponent implements Initializable {
    static final String WATCHED_STYLE = "watched";

    private final Episode media;
    private final LocaleText localeText;
    private final ImageService imageService;

    private boolean watched;
    /**
     * The action which needs to be invoked when the watched icon is clicked.
     * It invoked the consumer with the new expected value for the watched state.
     */
    @Setter
    private Consumer<Boolean> onWatchClicked;
    /**
     * The action to invoke when the episode component is being destroyed.
     */
    @Setter
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
        episodeNumber.setText(String.valueOf(media.episode()));
        title.setText(media.title());
        airDate.setText(localeText.get(DetailsMessage.AIR_DATE, ShowHelperService.AIRED_DATE_PATTERN.format(media.getAirDate())));
        synopsis.setText(media.synopsis());
        graphic.parentProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue == null && onDestroy != null) {
                onDestroy.run();
            }
        });

        loadAndUpdateImageArt();
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

    private void loadAndUpdateImageArt() {
        imageService.getArtPlaceholder()
                .thenCompose(image -> {
                    Platform.runLater(() -> episodeArt.setImage(image));
                    return media.getThumb()
                            .filter(url -> !url.isEmpty())
                            .map(imageService::load)
                            .orElse(CompletableFuture.completedFuture(image));
                })
                .whenComplete((image, throwable) -> {
                    if (throwable == null) {
                        Platform.runLater(() -> episodeArt.setImage(image));
                    } else {
                        log.error("Failed to load artwork, {}", throwable.getMessage(), throwable);
                    }
                });
    }

    @FXML
    void onWatchedClicked(MouseEvent event) {
        event.consume();
        if (onWatchClicked != null) {
            onWatchClicked.accept(!watched);
        }
    }
}
