package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.ui.events.ActivityManager;
import com.github.yoep.popcorn.ui.events.ClosePlayerEvent;
import com.github.yoep.popcorn.ui.events.PlayVideoEvent;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import lombok.RequiredArgsConstructor;

import javax.annotation.PostConstruct;

@RequiredArgsConstructor
public class PlayerHeaderComponent {
    private final ActivityManager activityManager;

    @FXML
    private Label title;

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializeActivityListeners();
    }

    private void initializeActivityListeners() {
        activityManager.register(PlayVideoEvent.class, this::onPlayVideo);
        activityManager.register(ClosePlayerEvent.class, this::onClose);
    }

    //endregion

    //region Functions

    private void onPlayVideo(PlayVideoEvent activity) {
        // set the title of the video as it should be always present
        Platform.runLater(() -> {
            this.title.setText(activity.getTitle());
        });
    }

    private void onClose(ClosePlayerEvent activity) {
        reset();
    }

    private void reset() {
        Platform.runLater(() -> title.setText(null));
    }

    //endregion
}
