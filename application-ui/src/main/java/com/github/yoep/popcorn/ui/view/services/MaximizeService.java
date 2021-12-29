package com.github.yoep.popcorn.ui.view.services;

import com.github.spring.boot.javafx.view.ViewManager;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.UISettings;
import javafx.application.Platform;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.ReadOnlyBooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.stage.Screen;
import javafx.stage.Stage;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;

@Slf4j
@Service
@RequiredArgsConstructor
public class MaximizeService {
    public static final String MAXIMIZED_PROPERTY = "maximized";

    private final ViewManager viewManager;
    private final SettingsService settingsService;

    private final BooleanProperty maximized = new SimpleBooleanProperty(this, MAXIMIZED_PROPERTY);

    private double originX;
    private double originY;
    private double originWidth;
    private double originHeight;

    //region Properties

    /**
     * Check if the stage is maximized or not.
     *
     * @return Returns true if the stage is maximized, else false.
     */
    public boolean isMaximized() {
        return maximized.get();
    }

    /**
     * Get the maximized property.
     *
     * @return Returns the maximized property.
     */
    public ReadOnlyBooleanProperty maximizedProperty() {
        return maximized;
    }

    /**
     * Set the maximized state of the stage.
     *
     * @param maximized The maximized value.
     */
    public void setMaximized(boolean maximized) {
        this.maximized.set(maximized);
    }

    //endregion

    //region Methods

    /**
     * Minimize the stage to the taskbar of the OS.
     */
    public void minimize() {
        viewManager.getPrimaryStage()
                .ifPresent(e -> e.setIconified(true));
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        log.trace("Initializing maximize service");
        initializeStageListeners();
        initializeMaximizedListener();
        log.trace("Maximize service initialized");
    }

    private void initializeStageListeners() {
        viewManager.primaryStageProperty().addListener((observable, oldValue, newValue) -> onStageChanged(newValue));
    }

    private void initializeMaximizedListener() {
        maximized.addListener((observable, oldValue, newValue) -> onMaximizedChanged(oldValue, newValue));
    }

    //endregion

    //region Functions

    private void onStageChanged(Stage stage) {
        if (stage == null)
            return;

        stage.maximizedProperty().addListener((observable, oldValue, newValue) -> maximized.setValue(newValue));
    }

    private void onMaximizedChanged(Boolean oldValue, boolean newValue) {
        var uiSettings = getUiSettings();

        // store the state in the settings
        log.trace("Stage maximized state is being changed from \"{}\" to \"{}\"", oldValue, newValue);
        uiSettings.setMaximized(newValue);

        // update the stage
        if (newValue) {
            toMaximizedStage();
        } else {
            toWindowedStage();
        }
    }

    private void toWindowedStage() {
        viewManager.getPrimaryStage().ifPresent(stage -> {
            stage.setX(originX);
            stage.setY(originY);
            stage.setWidth(originWidth);
            stage.setHeight(originHeight);
        });
    }

    private void toMaximizedStage() {
        viewManager.getPrimaryStage().ifPresent(stage -> Platform.runLater(() -> {
            var screen = Screen.getPrimary();

            // store the current windowed stage information
            originX = stage.getX();
            originY = stage.getY();
            originWidth = stage.getWidth();
            originHeight = stage.getHeight();

            // maximize the stage
            stage.setX(0);
            stage.setY(0);
            stage.setWidth(screen.getVisualBounds().getWidth());
            stage.setHeight(screen.getVisualBounds().getHeight());
        }));
    }

    private UISettings getUiSettings() {
        return settingsService.getSettings().getUiSettings();
    }

    //endregion
}
