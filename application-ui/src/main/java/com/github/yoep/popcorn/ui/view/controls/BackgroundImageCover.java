package com.github.yoep.popcorn.ui.view.controls;

import javafx.scene.CacheHint;
import javafx.scene.Node;
import javafx.scene.effect.GaussianBlur;
import javafx.scene.image.Image;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.StackPane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.ClassPathResource;

import java.text.MessageFormat;

/**
 * Background image which is blurred and has a shadow cover on top of it.
 */
@Slf4j
public class BackgroundImageCover extends AnchorPane {
    private static final String STYLE_CLASS = "background-image";
    private static final String COVER_STYLE_CLASS = "background-cover";
    private static final Image BACKGROUND_PLACEHOLDER = loadPlaceholder();

    private final ImageCover imageCover = new ImageCover();
    private final StackPane coverPane = new StackPane();

    //region Constructor

    public BackgroundImageCover() {
        init();
    }

    //endregion

    //region Methods

    /**
     * Set the background image for this background cover.
     * The background image will only be set if the image is not in error state.
     *
     * @param image The image to set as background.
     */
    public void setBackgroundImage(final Image image) {
        showBackgroundImage(image);
    }

    /**
     * Reset the background image to the default background placeholder.
     */
    public void reset() {
        if (BACKGROUND_PLACEHOLDER != null && !BACKGROUND_PLACEHOLDER.isError()) {
            showBackgroundImage(BACKGROUND_PLACEHOLDER);
        } else {
            // fallback to a black background
            resetImage();
        }
    }

    //endregion

    //region Functions

    private void init() {
        initializeImageView();
        initializeCoverPane();
        initializeBackgroundImage();
        reset();
    }

    private void initializeImageView() {
        imageCover.setEffect(new GaussianBlur(30));
    }

    private void initializeCoverPane() {
        coverPane.getStyleClass().add(COVER_STYLE_CLASS);
    }

    private void initializeBackgroundImage() {
        this.setCache(true);
        this.setCacheHint(CacheHint.SCALE_AND_ROTATE);
        this.getStyleClass().add(STYLE_CLASS);

        anchor(imageCover);
        anchor(coverPane);
        this.getChildren().addAll(imageCover, coverPane);
    }

    private void showBackgroundImage(final Image image) {
        imageCover.setImage(image);
    }

    private void resetImage() {
        this.imageCover.reset();
    }

    private void anchor(Node node) {
        AnchorPane.setTopAnchor(node, 0.0);
        AnchorPane.setRightAnchor(node, 0.0);
        AnchorPane.setBottomAnchor(node, 0.0);
        AnchorPane.setLeftAnchor(node, 0.0);
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

    //endregion
}
