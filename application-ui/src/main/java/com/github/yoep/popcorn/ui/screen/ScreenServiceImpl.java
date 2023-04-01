package com.github.yoep.popcorn.ui.screen;

import com.github.spring.boot.javafx.view.ViewManager;
import com.github.yoep.popcorn.backend.adapters.screen.ScreenService;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayerStoppedEvent;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.view.services.MaximizeService;
import javafx.application.Platform;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.ReadOnlyBooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyCombination;
import javafx.stage.Stage;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;

@Slf4j
@Service
@RequiredArgsConstructor
public class ScreenServiceImpl implements ScreenService {
    public static final String FULLSCREEN_PROPERTY = "fullscreen";

    private final ViewManager viewManager;
    private final ApplicationConfig applicationConfig;
    private final EventPublisher eventPublisher;
    private final MaximizeService maximizeService;

    private final BooleanProperty fullscreen = new SimpleBooleanProperty(this, FULLSCREEN_PROPERTY, false);

    private Stage primaryStage;
    private long lastChange;

    //region Properties

    @Override
    public boolean isFullscreen() {
        return fullscreen.get();
    }

    @Override
    public ReadOnlyBooleanProperty fullscreenProperty() {
        return fullscreen;
    }

    //endregion

    //region ScreenService

    @Override
    public void toggleFullscreen() {
        // check if no duplicate screen toggle command has been received
        if (System.currentTimeMillis() - lastChange < 300)
            return;

        Platform.runLater(() -> {
            lastChange = System.currentTimeMillis();
            primaryStage.setFullScreen(!primaryStage.isFullScreen());
        });
    }

    @Override
    public void fullscreen(final boolean isFullscreenEnabled) {
        Platform.runLater(() -> {
            lastChange = System.currentTimeMillis();
            primaryStage.setFullScreen(isFullscreenEnabled);
        });
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        initializeViewManagerListeners();
        eventPublisher.register(PlayerStoppedEvent.class, event -> {
            if (!applicationConfig.isKioskMode()) {
                Platform.runLater(() -> primaryStage.setFullScreen(false));
            }
            return event;
        });
    }

    private void initializeViewManagerListeners() {
        viewManager.primaryStageProperty().addListener((observable, oldValue, newValue) -> registerListener(newValue));
    }

    //endregion

    //region Functions

    private void registerListener(Stage primaryStage) {
        // store the primary screen for later use
        log.trace("Primary stage is being registered");
        this.primaryStage = primaryStage;

        if (applicationConfig.isKioskMode()) {
            log.trace("Kiosk mode is activated, disabling the fullscreen exit key");
            maximizeService.setMaximized(false);
            primaryStage.setFullScreenExitKeyCombination(KeyCombination.NO_MATCH);
            primaryStage.setFullScreen(true);
        } else {
            primaryStage.setFullScreenExitKeyCombination(KeyCombination.valueOf(KeyCode.F11.getName()));
        }

        fullscreen.bind(primaryStage.fullScreenProperty());
        fullscreen.addListener((observable, oldValue, newValue) -> lastChange = System.currentTimeMillis());
    }

    //endregion
}
