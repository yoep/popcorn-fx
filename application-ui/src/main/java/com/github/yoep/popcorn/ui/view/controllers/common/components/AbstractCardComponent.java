package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.image.Image;
import javafx.scene.layout.*;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Objects;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
public abstract class AbstractCardComponent implements Initializable {
    protected static final int POSTER_WIDTH = 201;
    protected static final int POSTER_HEIGHT = 294;

    protected final ImageService imageService;
    protected final Media media;

    protected AbstractCardComponent(ImageService imageService) {
        Objects.requireNonNull(imageService, "imageService cannot be null");
        this.imageService = imageService;
        this.media = null;
    }

    protected AbstractCardComponent(ImageService imageService, Media media) {
        Objects.requireNonNull(imageService, "imageService cannot be null");
        this.imageService = imageService;
        this.media = media;
    }

    @FXML
    Pane poster;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeImage();
    }

    protected void initializeImage() {
        // use the post holder as the default image while the media image is being loaded
        imageService.getPosterPlaceholder(POSTER_WIDTH, POSTER_HEIGHT)
                .whenComplete((posterPlaceholder, throwable) -> {
                    if (throwable == null) {
                        setBackgroundImage(posterPlaceholder, false);

                        var loadPosterFuture = imageService.loadPoster(media, POSTER_WIDTH, POSTER_HEIGHT);
                        handlePosterLoadFuture(loadPosterFuture);
                    } else {
                        log.error("Failed to load poster placeholder image", throwable);
                    }
                });
    }

    protected void handlePosterLoadFuture(CompletableFuture<Optional<Image>> loadPosterFuture) {
        loadPosterFuture.whenComplete((image, throwable) -> {
            if (throwable == null) {
                image.ifPresent(e -> Platform.runLater(() -> setBackgroundImage(e, true)));
            } else {
                log.error("Failed to load poster image, {}", throwable.getMessage(), throwable);
            }
        });
    }

    /**
     * Set the given image as poster node background image.
     *
     * @param image The image to use as background.
     * @param cover Whether the image should be sized to "cover" the Region
     */
    protected void setBackgroundImage(Image image, boolean cover) {
        BackgroundSize size = new BackgroundSize(BackgroundSize.AUTO, BackgroundSize.AUTO, false, false, false, cover);
        BackgroundImage backgroundImage = new BackgroundImage(image, BackgroundRepeat.NO_REPEAT, BackgroundRepeat.NO_REPEAT, BackgroundPosition.CENTER, size);
        poster.setBackground(new Background(backgroundImage));
    }
}
