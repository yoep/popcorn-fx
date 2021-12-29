package com.github.yoep.popcorn.ui.view.controllers.tv.sections;

import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.events.ShowSerieDetailsEvent;
import javafx.fxml.FXML;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.event.EventListener;

@Slf4j
public class DetailsSectionController {
    @FXML
    private Pane movieDetailsPane;
    @FXML
    private Pane showDetailsPane;

    //region Methods

    @EventListener(ShowMovieDetailsEvent.class)
    public void onShowMovieDetails() {
        switchContent(true);
    }

    @EventListener(ShowSerieDetailsEvent.class)
    public void onShowSerieDetails() {
        switchContent(false);
    }

    //endregion

    private void switchContent(boolean isMovieDetails) {
        movieDetailsPane.setVisible(isMovieDetails);
        showDetailsPane.setVisible(!isMovieDetails);
    }
}
