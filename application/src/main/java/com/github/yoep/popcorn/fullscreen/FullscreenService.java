package com.github.yoep.popcorn.fullscreen;

import com.github.spring.boot.javafx.view.ViewManager;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.FullscreenActivity;
import com.github.yoep.popcorn.activities.ClosePlayerActivity;
import com.github.yoep.popcorn.activities.ToggleFullscreenActivity;
import javafx.application.Platform;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyCombination;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;

@Slf4j
@Service
@RequiredArgsConstructor
public class FullscreenService {
    private final ActivityManager activityManager;
    private final ViewManager viewManager;

    private boolean listenerRegistered;
    private long lastChange;

    @PostConstruct
    private void init() {
        activityManager.register(ToggleFullscreenActivity.class, activity -> onToggleFullscreen());
        activityManager.register(ClosePlayerActivity.class, activity -> onClosePlayer());
    }

    private void onToggleFullscreen() {
        viewManager.getPrimaryStage()
                .ifPresent(stage -> {
                    if (!listenerRegistered)
                        registerListener();

                    // check if no duplicate screen toggle command has been received
                    if (System.currentTimeMillis() - lastChange < 300)
                        return;

                    Platform.runLater(() -> {
                        lastChange = System.currentTimeMillis();
                        stage.setFullScreen(!stage.isFullScreen());
                    });
                });
    }

    private void onClosePlayer() {
        viewManager.getPrimaryStage()
                .ifPresent(stage -> Platform.runLater(() -> stage.setFullScreen(false)));
    }

    private void registerListener() {
        listenerRegistered = true;

        viewManager.getPrimaryStage()
                .ifPresent(stage -> {
                    stage.setFullScreenExitKeyCombination(KeyCombination.valueOf(KeyCode.F11.getName()));
                    stage.fullScreenProperty().addListener((observable, oldValue, newValue) -> {
                        lastChange = System.currentTimeMillis();
                        activityManager.register((FullscreenActivity) () -> newValue);
                    });
                });
    }
}
