package com.github.yoep.popcorn.view.controllers.desktop.components;

import javafx.fxml.FXML;
import javafx.scene.image.Image;
import javafx.scene.layout.*;
import org.springframework.core.io.ClassPathResource;

public abstract class AbstractCardComponent {
    protected static final int POSTER_WIDTH = 134;
    protected static final int POSTER_HEIGHT = 196;

    private static final String POSTER_HOLDER = "/images/posterholder.png";

    @FXML
    private Pane poster;

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
        BackgroundSize size = new BackgroundSize(BackgroundSize.AUTO, BackgroundSize.AUTO, false, false, true, cover);
        BackgroundImage backgroundImage = new BackgroundImage(image, BackgroundRepeat.NO_REPEAT, BackgroundRepeat.NO_REPEAT, BackgroundPosition.CENTER, size);
        poster.setBackground(new Background(backgroundImage));
    }
}
