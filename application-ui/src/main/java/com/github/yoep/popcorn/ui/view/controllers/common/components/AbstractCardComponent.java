package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.image.Image;
import javafx.scene.layout.*;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.ClassPathResource;

import java.net.URL;
import java.text.MessageFormat;
import java.util.Objects;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
public abstract class AbstractCardComponent implements Initializable {
    private static final Image POSTER_HOLDER_IMAGE = loadPosterHolderImage();

    protected static final int POSTER_WIDTH = 201;
    protected static final int POSTER_HEIGHT = 294;

    private static final String POSTER_HOLDER = "/images/posterholder.png";

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
        setPosterHolderImage();

        var loadPosterFuture = imageService.loadPoster(media, POSTER_WIDTH, POSTER_HEIGHT);
        handlePosterLoadFuture(loadPosterFuture);
    }

    protected void setPosterHolderImage() {
        try {
            // use the post holder as the default image while the media image is being loaded
            setBackgroundImage(POSTER_HOLDER_IMAGE, false);
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    protected void handlePosterLoadFuture(CompletableFuture<Optional<Image>> loadPosterFuture) {
        loadPosterFuture.whenComplete((image, throwable) -> {
            if (throwable == null) {
                image.ifPresent(e -> setBackgroundImage(e, true));
            } else {
                log.error(throwable.getMessage(), throwable);
            }
        });
    }

    /**
     * Get the poster holder image resource.
     *
     * @return Returns the image resource.
     */
    protected static ClassPathResource getPosterHolderResource() {
        return new ClassPathResource(POSTER_HOLDER);
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

    private static Image loadPosterHolderImage() {
        try {
            var inputStream = getPosterHolderResource().getInputStream();
            var image = new Image(inputStream);

            if (!image.isError()) {
                return image;
            } else {
                handleImageError(image);
            }
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }

        return null;
    }

    private static void handleImageError(Image image) {
        var exception = image.getException();
        var message = MessageFormat.format("Failed to load image card poster url \"{0}\", {1}", image.getUrl(), exception.getMessage());

        log.warn(message, exception);
    }
}
