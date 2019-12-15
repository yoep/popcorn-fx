package com.github.yoep.popcorn.controllers;

import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.PlayMediaActivity;
import com.github.yoep.popcorn.activities.PlayerCloseActivity;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Controller;

import java.net.URL;
import java.util.ResourceBundle;

@Controller
@RequiredArgsConstructor
public class MainController implements Initializable {
    private final ActivityManager activityManager;

    @FXML
    private Pane contentSection;
    @FXML
    private Pane playerSection;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        playerSection.setVisible(false);
        initializeListeners();
    }

    private void initializeListeners() {
        activityManager.register(PlayMediaActivity.class, activity -> switchSection(true));
        activityManager.register(PlayerCloseActivity.class, activity -> switchSection(false));
    }

    private void switchSection(final boolean showPlayer) {
        Platform.runLater(() -> {
            contentSection.setVisible(!showPlayer);
            playerSection.setVisible(showPlayer);
        });
    }
}
