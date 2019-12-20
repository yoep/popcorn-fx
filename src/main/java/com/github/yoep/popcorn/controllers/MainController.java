package com.github.yoep.popcorn.controllers;

import com.github.yoep.popcorn.activities.*;
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
    private Pane settingsSection;
    @FXML
    private Pane playerSection;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        playerSection.setVisible(false);
        settingsSection.setVisible(false);
        initializeListeners();
    }

    private void initializeListeners() {
        activityManager.register(PlayMediaActivity.class, activity -> switchSection(ActiveSection.PLAYER));
        activityManager.register(ShowSettingsActivity.class, activity -> switchSection(ActiveSection.SETTINGS));
        activityManager.register(CloseSettingsActivity.class, activity -> switchSection(ActiveSection.CONTENT));
        activityManager.register(PlayerCloseActivity.class, activity -> switchSection(ActiveSection.CONTENT));
    }

    private void switchSection(final ActiveSection activeSection) {
        Platform.runLater(() -> {
            contentSection.setVisible(activeSection == ActiveSection.CONTENT);
            settingsSection.setVisible(activeSection == ActiveSection.SETTINGS);
            playerSection.setVisible(activeSection == ActiveSection.PLAYER);
        });
    }

    private enum ActiveSection {
        CONTENT,
        SETTINGS,
        PLAYER
    }
}
