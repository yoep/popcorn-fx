package com.github.yoep.popcorn.view.controllers.desktop.sections;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.ShowMovieDetailsActivity;
import com.github.yoep.popcorn.activities.ShowSerieDetailsActivity;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@ViewController
@RequiredArgsConstructor
public class DetailsSectionController implements Initializable {
    private final ActivityManager activityManager;

    @FXML
    private Pane movieDetailsPane;
    @FXML
    private Pane showDetailsPane;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        activityManager.register(ShowMovieDetailsActivity.class, activity -> switchContent(true));
        activityManager.register(ShowSerieDetailsActivity.class, activity -> switchContent(false));
    }

    private void switchContent(boolean isMovieDetails) {
        movieDetailsPane.setVisible(isMovieDetails);
        showDetailsPane.setVisible(!isMovieDetails);
    }
}
