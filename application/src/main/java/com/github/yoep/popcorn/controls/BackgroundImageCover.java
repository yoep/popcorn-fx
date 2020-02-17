package com.github.yoep.popcorn.controls;

import com.github.yoep.popcorn.media.providers.models.Images;
import com.github.yoep.popcorn.media.providers.models.Media;
import javafx.geometry.Insets;
import javafx.scene.CacheHint;
import javafx.scene.effect.GaussianBlur;
import javafx.scene.image.Image;
import javafx.scene.layout.*;
import javafx.scene.paint.Color;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.util.Optional;

/**
 * Background image which is blurred and has a shadow cover on top of it.
 */
@Slf4j
public class BackgroundImageCover extends StackPane {
    private static final String STYLE_CLASS = "background-image";
    private static final String COVER_STYLE_CLASS = "background-cover";

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
     * Reset the background image to a black region.
     */
    public void reset() {
        this.imagePane.setBackground(new Background(new BackgroundFill(Color.BLACK, CornerRadii.EMPTY, Insets.EMPTY)));
    }

    private void init() {
        initializeImagePane();
        initializeCoverPane();
        initializeBackgroundImage();
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
        final BackgroundSize backgroundSize =
                new BackgroundSize(BackgroundSize.AUTO, BackgroundSize.AUTO, false, false, true, true);
        final BackgroundImage backgroundImage =
                new BackgroundImage(image, BackgroundRepeat.NO_REPEAT, BackgroundRepeat.NO_REPEAT, BackgroundPosition.DEFAULT, backgroundSize);

        this.imagePane.setBackground(new Background(backgroundImage));
    }
}
