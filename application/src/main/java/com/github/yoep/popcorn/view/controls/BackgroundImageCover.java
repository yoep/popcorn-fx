package com.github.yoep.popcorn.view.controls;

import com.github.yoep.popcorn.media.providers.models.Images;
import com.github.yoep.popcorn.media.providers.models.Media;
import javafx.geometry.Insets;
import javafx.scene.CacheHint;
import javafx.scene.effect.GaussianBlur;
import javafx.scene.image.Image;
import javafx.scene.layout.*;
import javafx.scene.paint.Color;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.ClassPathResource;
import org.springframework.util.Assert;

import java.util.Optional;

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
     * Load the background image for the given {@link Media} item.
     *
     * @param media The media item to show the background image of.
     */
    public void load(final Media media) {
        Assert.notNull(media, "media cannot be null");

        // always set a black background
        reset();

        // try to load the background image
        try {
            Optional.ofNullable(media.getImages())
                    .map(Images::getFanart)
                    .filter(e -> !e.equalsIgnoreCase("n/a"))
                    .map(url -> new Image(url, true))
                    .ifPresent(this::showBackgroundImage);
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    /**
     * Reset the background image to the default background placeholder.
     */
    public void reset() {
        if (BACKGROUND_PLACEHOLDER != null) {
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
        var backgroundSize = new BackgroundSize(BackgroundSize.AUTO, BackgroundSize.AUTO, false, false, false, true);
        var backgroundImage = new BackgroundImage(image, BackgroundRepeat.NO_REPEAT, BackgroundRepeat.NO_REPEAT, BackgroundPosition.CENTER, backgroundSize);

        this.imagePane.setBackground(new Background(backgroundImage));
    }

    private void resetToBlackBackground() {
        this.imagePane.setBackground(new Background(new BackgroundFill(Color.BLACK, CornerRadii.EMPTY, Insets.EMPTY)));
    }

    private static Image loadPlaceholder() {
        try {
            var resource = new ClassPathResource("/images/placeholder-background.jpg");
            return new Image(resource.getInputStream());
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }

        return null;
    }
}
