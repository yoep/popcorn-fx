package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.view.ViewManager;
import javafx.application.Platform;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.ReadOnlyBooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.stage.Screen;
import javafx.stage.Stage;
import lombok.extern.slf4j.Slf4j;

import java.util.concurrent.CompletableFuture;

@Slf4j
public class MaximizeService {
    public static final String MAXIMIZED_PROPERTY = "maximized";

    private final ViewManager viewManager;
    private final ApplicationConfig applicationConfig;

    private final BooleanProperty maximized = new SimpleBooleanProperty(this, MAXIMIZED_PROPERTY);

    private double originX;
    private double originY;
    private double originWidth;
    private double originHeight;

    public MaximizeService(ViewManager viewManager, ApplicationConfig applicationConfig) {
        this.viewManager = viewManager;
        this.applicationConfig = applicationConfig;
        init();
    }

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
        // store the state in the settings
        log.trace("Stage maximized state is being changed from \"{}\" to \"{}\"", oldValue, newValue);
        getUiSettings().whenComplete((settings, throwable) -> {
            if (throwable == null) {
                applicationConfig.update(ApplicationSettings.UISettings.newBuilder(settings)
                        .setMaximized(newValue)
                        .build());
            } else {
                log.error("Failed to retrieve settings", throwable);
            }
        });

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
        viewManager.getPrimaryStage().ifPresentOrElse(
                stage -> Platform.runLater(() -> {
                    var screen = detectCurrentScreen(stage);

                    // store the current windowed stage information
                    originX = stage.getX();
                    originY = stage.getY();
                    originWidth = stage.getWidth();
                    originHeight = stage.getHeight();

                    // maximize the stage
                    stage.setX(screen.getVisualBounds().getMinX());
                    stage.setY(screen.getVisualBounds().getMinY());
                    stage.setWidth(screen.getVisualBounds().getWidth());
                    stage.setHeight(screen.getVisualBounds().getHeight());
                }),
                () -> log.error("Unable to update maximize state, primary stage not found"));
    }

    private CompletableFuture<ApplicationSettings.UISettings> getUiSettings() {
        return applicationConfig.getSettings().thenApply(ApplicationSettings::getUiSettings);
    }

    private static Screen detectCurrentScreen(Stage stage) {
        var screens = Screen.getScreensForRectangle(stage.getX(), stage.getY(), stage.getWidth(), stage.getHeight());

        return screens.stream()
                .findFirst()
                .orElseGet(() -> {
                    log.warn("Failed to detect current window screen, using primary screen instead");
                    return Screen.getPrimary();
                });
    }

    //endregion
}
