package com.github.yoep.popcorn.services;

import com.github.spring.boot.javafx.view.ViewManager;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.FullscreenActivity;
import com.github.yoep.popcorn.activities.ToggleFullscreenActivity;
import javafx.application.Platform;
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

    @PostConstruct
    private void init() {
        activityManager.register(ToggleFullscreenActivity.class, activity -> viewManager.getPrimaryStage()
                .ifPresent(stage -> Platform.runLater(() -> {
                    stage.setFullScreen(!stage.isFullScreen());
                    activityManager.register((FullscreenActivity) stage::isFullScreen);
                })));
    }
}
