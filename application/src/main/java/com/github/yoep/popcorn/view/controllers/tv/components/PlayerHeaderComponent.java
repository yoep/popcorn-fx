package com.github.yoep.popcorn.view.controllers.tv.components;

import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.ClosePlayerActivity;
import com.github.yoep.popcorn.activities.PlayVideoActivity;
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
        activityManager.register(PlayVideoActivity.class, this::onPlayVideo);
        activityManager.register(ClosePlayerActivity.class, this::onClose);
    }

    //endregion

    //region Functions

    private void onPlayVideo(PlayVideoActivity activity) {
        // set the title of the video as it should be always present
        Platform.runLater(() -> {
            this.title.setText(activity.getTitle());
        });
    }

    private void onClose(ClosePlayerActivity activity) {
        reset();
    }

    private void reset() {
        Platform.runLater(() -> title.setText(null));
    }

    //endregion
}
