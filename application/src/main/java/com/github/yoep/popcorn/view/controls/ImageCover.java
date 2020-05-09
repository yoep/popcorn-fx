package com.github.yoep.popcorn.view.controls;

import javafx.beans.value.ChangeListener;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import javafx.scene.shape.Rectangle;
import lombok.extern.slf4j.Slf4j;

import java.text.MessageFormat;

/**
 * Displays an image which fully covers the pane size.
 */
@Slf4j
public class ImageCover extends AnchorPane {
    private static final String STYLE_CLASS = "image-cover";

    private final ChangeListener<Number> parentWidthListener = (observable, oldValue, newValue) -> onParentWidthChanged(newValue);
    private final ChangeListener<Number> parentHeightListener = (observable, oldValue, newValue) -> onParentHeightChanged(newValue);

    private final ImageView imageView = new ImageView();
    private final Rectangle clipView = new Rectangle();

    //region Constructors

    public ImageCover() {
        init();
    }

    //endregion

    //region Methods

    /**
     * Set the image which needs to cover the pane.
     *
     * @param image The image to cover the pane with.
     */
    public void setImage(final Image image) {
        if (image != null) {
            if (!image.isError()) {
                imageView.setImage(image);
            } else {
                handleImageError(image);
            }
        } else {
            reset();
        }
    }

    /**
     * Reset the image cover to a blank canvas.
     */
    public void reset() {
        // remove the image
        imageView.setImage(null);
    }

    //endregion

    //region Functions

    private void init() {
        initializeAnchorPane();
        initializeImageView();
        initializeListeners();
        initializeCover();
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
        imageView.setManaged(false);
        imageView.setPreserveRatio(true);
        imageView.setClip(clipView);
        imageView.imageProperty().addListener((observable, oldValue, newValue) -> resizeImage());
    }

    private void initializeListeners() {
        this.widthProperty().addListener((observable, oldValue, newValue) -> resizeImage());
        this.heightProperty().addListener((observable, oldValue, newValue) -> resizeImage());
    }

    private void initializeCover() {
        getStyleClass().add(STYLE_CLASS);

        getChildren().add(imageView);
    }

    private void onParentWidthChanged(Number newValue) {
        this.setMaxWidth(newValue.doubleValue());
        this.setMinWidth(newValue.doubleValue());
        this.setPrefWidth(newValue.doubleValue());

        clipView.setWidth(newValue.longValue());
        requestLayout();
    }

    private void onParentHeightChanged(Number newValue) {
        this.setMaxHeight(newValue.doubleValue());
        this.minHeight(newValue.doubleValue());
        this.setPrefHeight(newValue.doubleValue());

        clipView.setHeight(newValue.longValue());
        requestLayout();
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

    private static void handleImageError(Image image) {
        var exception = image.getException();
        var message = MessageFormat.format("Failed to load image cover for url \"{0}\", {1}", image.getUrl(), exception.getMessage());

        log.warn(message, exception);
    }

    //endregion
}
