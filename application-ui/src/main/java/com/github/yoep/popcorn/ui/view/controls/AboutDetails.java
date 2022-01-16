package com.github.yoep.popcorn.ui.view.controls;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.ui.view.model.AboutDetail;
import javafx.scene.control.Label;
import javafx.scene.layout.GridPane;

import java.util.List;

public class AboutDetails extends GridPane {
    private static final String STYLE_CLASS = "about-details";

    public AboutDetails() {
        init();
    }

    //region Properties

    public void setItems(List<AboutDetail> items) {
        renderItems(items);
    }

    //endregion

    private void init() {
        this.getStyleClass().add(STYLE_CLASS);
    }

    private void renderItems(List<AboutDetail> items) {
        this.getChildren().clear();

        items.forEach(e -> {
            var nameLabel = new Label(e.getName());
            var descriptionLabel = new Label(e.getDescription().orElse(null));
            var stateIcon = stateToIcon(e.getState());

            this.addRow(this.getRowCount(), nameLabel, descriptionLabel, stateIcon);
        });
    }

    private static Icon stateToIcon(AboutDetail.State state) {
        switch (state) {
            case UNKNOWN:
                return new Icon(Icon.QUESTION_CIRCLE_UNICODE);
            case READY:
                return new Icon(Icon.CHECK_CIRCLE_UNICODE);
            case ERROR:
            default:
                return new Icon(Icon.TIMES_CIRCLE_O_UNICODE);
        }
    }

}
