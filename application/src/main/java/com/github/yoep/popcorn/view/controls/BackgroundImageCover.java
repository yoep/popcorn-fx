package com.github.yoep.popcorn.view.controls;

import javafx.beans.value.ChangeListener;
import javafx.scene.CacheHint;
import javafx.scene.effect.GaussianBlur;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
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

    private final ChangeListener<Number> parentWidthListener = (observable, oldValue, newValue) -> onParentWidthChanged(newValue);
    private final ChangeListener<Number> parentHeightListener = (observable, oldValue, newValue) -> onParentHeightChanged(newValue);

    private final ImageView imageView = new ImageView();
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
            resetImage();
        }
    }

    //endregion

    //region Functions

    private void init() {
        initializeAnchorPane();
        initializeImageView();
        initializeCoverPane();
        initializeBackgroundImage();
        reset();
    }

    private void initializeAnchorPane() {
        parentProperty().addListener((observable, oldValue, newValue) -> {
            if (oldValue != null) {
                var parent = (Pane) oldValue;

                parent.widthProperty().removeListener(parentWidthListener);
                parent.heightProperty().removeListener(parentHeightListener);
            }

            if (newValue != null) {
                var parent = (Pane) newValue;

                parent.widthProperty().addListener(parentWidthListener);
                parent.heightProperty().addListener(parentHeightListener);
            }
        });
    }

    private void initializeImageView() {
        imageView.setPreserveRatio(true);
        imageView.setEffect(new GaussianBlur(30));
        imageView.imageProperty().addListener((observable, oldValue, newValue) -> resizeImage());

        this.widthProperty().addListener((observable, oldValue, newValue) -> resizeImage());
        this.heightProperty().addListener((observable, oldValue, newValue) -> resizeImage());
    }

    private void initializeCoverPane() {
        coverPane.getStyleClass().add(COVER_STYLE_CLASS);

        AnchorPane.setTopAnchor(coverPane, 0.0);
        AnchorPane.setRightAnchor(coverPane, 0.0);
        AnchorPane.setBottomAnchor(coverPane, 0.0);
        AnchorPane.setLeftAnchor(coverPane, 0.0);
    }

    private void initializeBackgroundImage() {
        this.setCache(true);
        this.setCacheHint(CacheHint.SCALE_AND_ROTATE);
        this.getStyleClass().add(STYLE_CLASS);

        this.getChildren().addAll(imageView, coverPane);
    }

    private void onParentWidthChanged(Number newValue) {
        this.setMaxWidth(newValue.doubleValue());
        this.setMinWidth(newValue.doubleValue());
        this.setPrefWidth(newValue.doubleValue());
    }

    private void onParentHeightChanged(Number newValue) {
        this.setMaxHeight(newValue.doubleValue());
        this.setMinWidth(newValue.doubleValue());
        this.setPrefHeight(newValue.doubleValue());
    }

    private void showBackgroundImage(final Image image) {
        if (!image.isError()) {
            imageView.setImage(image);
        } else {
            handleImageError(image);
        }
    }

    private void resetImage() {
        this.imageView.setImage(null);
    }

    private void resizeImage() {
        var image = imageView.getImage();
        var paneWidth = this.getWidth();
        var paneHeight = this.getHeight();

        // check if we need to recalculate the image size
        if (image == null || paneWidth == 0 || paneHeight == 0)
            return;

        var imageWidth = image.getWidth();
        var imageHeight = image.getHeight();
        var scale = Math.max(paneWidth / imageWidth, paneHeight / imageHeight);
        var fitWidth = imageWidth * scale;
        var fitHeight = imageHeight * scale;

        imageView.setFitWidth(fitWidth);
        imageView.setFitHeight(fitHeight);
        imageView.setX((paneWidth - fitWidth) / 2);
        imageView.setY((paneHeight - fitHeight) / 2);
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
