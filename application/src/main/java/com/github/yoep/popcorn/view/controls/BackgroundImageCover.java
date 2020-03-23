package com.github.yoep.popcorn.view.controls;

import javafx.geometry.Insets;
import javafx.scene.CacheHint;
import javafx.scene.effect.GaussianBlur;
import javafx.scene.image.Image;
import javafx.scene.layout.*;
import javafx.scene.paint.Color;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.ClassPathResource;

import java.text.MessageFormat;

/**
 * Background image which is blurred and has a shadow cover on top of it.
 */
@Slf4j
public class BackgroundImageCover extends StackPane {
    private static final String STYLE_CLASS = "background-image";
    private static final String COVER_STYLE_CLASS = "background-cover";
    private static final Image BACKGROUND_PLACEHOLDER = loadPlaceholder();

    private final BorderPane imagePane;
    private final BorderPane coverPane;

    public BackgroundImageCover() {
        this.imagePane = new BorderPane();
        this.coverPane = new BorderPane();
        init();
    }

    /**
     * Set the background image for this background cover.
     * The background image will only be set if the image is not in error state.
     *
     * @param image The image to set as background.
     */
    public void setBackgroundImage(final Image image) {
        if (!image.isError()) {
            showBackgroundImage(image);
        } else {
            handleImageError(image);
        }
    }

    /**
     * Reset the background image to the default background placeholder.
     */
    public void reset() {
        if (BACKGROUND_PLACEHOLDER != null && !BACKGROUND_PLACEHOLDER.isError()) {
            showBackgroundImage(BACKGROUND_PLACEHOLDER);
        } else {
            // fallback to a black background
            resetToBlackBackground();
        }
    }

    private void init() {
        initializeImagePane();
        initializeCoverPane();
        initializeBackgroundImage();
        reset();
    }

    private void initializeImagePane() {
        imagePane.setEffect(new GaussianBlur(30));
    }

    private void initializeCoverPane() {
        coverPane.getStyleClass().add(COVER_STYLE_CLASS);
    }

    private void initializeBackgroundImage() {
        this.setCache(true);
        this.setCacheHint(CacheHint.SCALE_AND_ROTATE);
        this.getStyleClass().add(STYLE_CLASS);

        this.getChildren().addAll(imagePane, coverPane);
    }

    private void showBackgroundImage(final Image image) {
        if (!image.isError()) {
            var backgroundSize = new BackgroundSize(BackgroundSize.AUTO, BackgroundSize.AUTO, false, false, false, true);
            var backgroundImage = new BackgroundImage(image, BackgroundRepeat.NO_REPEAT, BackgroundRepeat.NO_REPEAT, BackgroundPosition.CENTER, backgroundSize);

            this.imagePane.setBackground(new Background(backgroundImage));
        } else {
            handleImageError(image);
        }
    }

    private void resetToBlackBackground() {
        this.imagePane.setBackground(new Background(new BackgroundFill(Color.BLACK, CornerRadii.EMPTY, Insets.EMPTY)));
    }

    private static Image loadPlaceholder() {
        try {
            var resource = new ClassPathResource("/images/placeholder-background.jpg");
            var image = new Image(resource.getInputStream());

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
        var message = MessageFormat.format("Failed to load background image cover for url \"{0}\", {1}", image.getUrl(), exception.getMessage());

        log.warn(message, exception);
    }
}
