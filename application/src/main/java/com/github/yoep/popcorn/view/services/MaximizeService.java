package com.github.yoep.popcorn.view.services;

import com.github.spring.boot.javafx.view.ViewManager;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.settings.models.UISettings;
import javafx.beans.value.ChangeListener;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;

@Slf4j
@Service
@RequiredArgsConstructor
public class MaximizeService {
    private final ViewManager viewManager;
    private final SettingsService settingsService;

    @PostConstruct
    private void init() {
        log.trace("Initializing maximize service");
        initializeStageListeners();
        log.trace("Maximize service initialized");
    }

    private void initializeStageListeners() {
        viewManager.primaryStageProperty().addListener((observable, oldValue, newValue) ->
                newValue.maximizedProperty().addListener(createMaximizedListener()));
    }

    private ChangeListener<Boolean> createMaximizedListener() {
        return (observable, oldValue, newValue) -> {
            var uiSettings = getUiSettings();

            log.trace("Stage maximized state is being changed from \"{}\" to \"{}\"", oldValue, newValue);
            uiSettings.setMaximized(newValue);
        };
    }

    private UISettings getUiSettings() {
        return settingsService.getSettings().getUiSettings();
    }
}
