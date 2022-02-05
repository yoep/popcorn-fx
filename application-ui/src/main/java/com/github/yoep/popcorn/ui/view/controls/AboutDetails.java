package com.github.yoep.popcorn.ui.view.controls;

import com.github.yoep.popcorn.backend.info.ComponentInfo;
import javafx.scene.layout.VBox;

import java.util.List;

public class AboutDetails extends VBox {
    private static final String STYLE_CLASS = "about-details";

    public AboutDetails() {
        init();
    }

    //region Properties

    public void setItems(List<ComponentInfo> items) {
        renderItems(items);
    }

    //endregion

    private void init() {
        initializeStyle();
    }

    private void initializeStyle() {
        this.getStyleClass().add(STYLE_CLASS);
    }

    private void renderItems(List<ComponentInfo> items) {
        this.getChildren().clear();

        items.forEach(e -> {
            var card = new AboutCard(e);
            this.getChildren().add(card);
        });
    }
}
