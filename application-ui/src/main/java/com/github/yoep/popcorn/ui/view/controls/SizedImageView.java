package com.github.yoep.popcorn.ui.view.controls;

import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.layout.Pane;

/**
 * The sized image view follows the size of its parent.
 */
public class SizedImageView extends ImageView {
    public SizedImageView() {
        init();
    }

    public SizedImageView(String url) {
        super(url);
        init();
    }

    public SizedImageView(Image image) {
        super(image);
        init();
    }

    private void init() {
        parentProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue != null) {
                bindToParent();
            }
        });
    }

    private void bindToParent() {
        if (getParent() instanceof Pane pane) {
            fitWidthProperty().bind(pane.widthProperty());
            fitHeightProperty().bind(pane.heightProperty());
        }
    }
}
