package com.github.yoep.popcorn.controllers.sections;

import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.LoadMovieActivity;
import com.github.yoep.popcorn.activities.PlayVideoActivity;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Controller;

import javax.annotation.PostConstruct;

@Controller
@RequiredArgsConstructor
public class PlayerSectionController {
    private final ActivityManager activityManager;

    @FXML
    private Pane player;
    @FXML
    private Pane loader;

    @PostConstruct
    private void init() {
        activityManager.register(PlayVideoActivity.class, activity -> switchContent(true));
        activityManager.register(LoadMovieActivity.class, activity -> switchContent(false));
    }

    private void switchContent(final boolean isPlayerVisible) {
        Platform.runLater(() -> {
            player.setVisible(isPlayerVisible);
            loader.setVisible(!isPlayerVisible);
        });
    }
}
