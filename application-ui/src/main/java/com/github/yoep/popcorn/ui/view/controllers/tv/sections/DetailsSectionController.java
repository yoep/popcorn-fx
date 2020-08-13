package com.github.yoep.popcorn.ui.view.controllers.tv.sections;

import com.github.yoep.popcorn.ui.events.ActivityManager;
import com.github.yoep.popcorn.ui.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.ui.events.ShowSerieDetailsEvent;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class DetailsSectionController implements Initializable {
    private final ActivityManager activityManager;

    @FXML
    private Pane movieDetailsPane;
    @FXML
    private Pane showDetailsPane;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        activityManager.register(ShowMovieDetailsEvent.class, activity -> switchContent(true));
        activityManager.register(ShowSerieDetailsEvent.class, activity -> switchContent(false));
    }

    private void switchContent(boolean isMovieDetails) {
        movieDetailsPane.setVisible(isMovieDetails);
        showDetailsPane.setVisible(!isMovieDetails);
    }
}
