package com.github.yoep.popcorn.view.controllers.common.sections;

import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.settings.models.StartScreen;
import com.github.yoep.popcorn.settings.models.UISettings;
import javafx.application.Platform;
import javafx.scene.Node;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

@Slf4j
public abstract class AbstractFilterSectionController {
    protected final SettingsService settingsService;

    private boolean startScreenInitialized;

    protected AbstractFilterSectionController(SettingsService settingsService) {
        Assert.notNull(settingsService, "settingsService cannot be null");
        this.settingsService = settingsService;
    }

    /**
     * Initialize the scene listener for this {@link AbstractFilterSectionController}.
     *
     * @param rootNode The root node to attach the scene listener to.
     */
    protected void initializeSceneListener(Node rootNode) {
        Assert.notNull(rootNode, "rootNode cannot be null");
        rootNode.sceneProperty().addListener((observable, oldValue, newValue) -> Platform.runLater(() -> {
            if (!startScreenInitialized) {
                var startScreen = getUiSettings().getStartScreen();

                initializeStartScreen(startScreen);
                startScreenInitialized = true;
            }
        }));
    }

    /**
     * Initialize the start screen of the application.
     *
     * @param startScreen The start screen that needs to be started.
     */
    protected abstract void initializeStartScreen(StartScreen startScreen);

    /**
     * Get the UI settings from the application.
     *
     * @return Returns the ui settings.
     */
    protected UISettings getUiSettings() {
        return settingsService.getSettings().getUiSettings();
    }
}
