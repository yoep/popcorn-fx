package com.github.yoep.popcorn.ui;

import com.github.spring.boot.javafx.SpringJavaFXApplication;
import com.github.spring.boot.javafx.stage.BorderlessStageWrapper;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.spring.boot.javafx.view.ViewManager;
import com.github.spring.boot.javafx.view.ViewManagerPolicy;
import com.github.spring.boot.javafx.view.ViewProperties;
import com.github.yoep.popcorn.backend.BackendConstants;
import com.github.yoep.popcorn.backend.settings.OptionsService;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.ui.stage.BorderlessStageHolder;
import com.github.yoep.popcorn.ui.view.services.MaximizeService;
import javafx.application.ConditionalFeature;
import javafx.application.Platform;
import javafx.scene.paint.Color;
import javafx.stage.Stage;
import javafx.stage.StageStyle;
import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.autoconfigure.SpringBootApplication;

import java.io.File;

@Slf4j
@SpringBootApplication
public class PopcornTimeApplication extends SpringJavaFXApplication {
    public static final String ICON_NAME = "icon_64.png";
    public static final String APPLICATION_TITLE = "Popcorn Time";

    public static void main(String[] args) {
        System.setProperty("log.dir", getLogDirectory());
        launch(PopcornTimeApplication.class, PopcornTimePreloader.class, args);
    }

    @Override
    public void start(Stage stage) throws Exception {
        log.trace("Starting the application");
        BorderlessStageHolder.setWrapper(new BorderlessStageWrapper(stage));
        super.start(stage);

        var loader = applicationContext.getBean(ViewLoader.class);
        var viewManager = applicationContext.getBean(ViewManager.class);

        initializeStage(stage);

        log.trace("Loading the main view of the application");
        loader.show(stage, "main.fxml", getViewProperties());
        viewManager.setPolicy(ViewManagerPolicy.CLOSEABLE);
    }

    private void initializeStage(Stage primaryStage) {
        log.trace("Initializing the primary stage");
        primaryStage.initStyle(StageStyle.UNDECORATED);
    }

    private ViewProperties getViewProperties() {
        log.trace("Building the view properties of the application");
        var optionsService = applicationContext.getBean(OptionsService.class);
        var maximizeService = applicationContext.getBean(MaximizeService.class);
        var options = optionsService.options();
        var properties = ViewProperties.builder()
                .title(APPLICATION_TITLE)
                .icon(ICON_NAME)
                .background(getBackgroundColor());

        // check if the big-picture or kiosk mode or maximized is enabled
        // if so, force the application to be maximized
        if (options.isBigPictureMode() || options.isKioskMode() || options.isMaximized()) {
            maximizeService.setMaximized(true);
        } else {
            var settingsService = applicationContext.getBean(SettingsService.class);
            var uiSettings = settingsService.getSettings().getUiSettings();

            maximizeService.setMaximized(uiSettings.isMaximized());
        }

        // check if the kiosk mode is enabled
        // if so, prevent the application from being resized
        if (options.isKioskMode()) {
            properties.resizable(false);
        }

        var viewProperties = properties.build();
        log.debug("Using the following view properties for the application: {}", viewProperties);
        return viewProperties;
    }

    private Color getBackgroundColor() {
        return Platform.isSupported(ConditionalFeature.TRANSPARENT_WINDOW) ? Color.TRANSPARENT : Color.BLACK;
    }

    public static String getLogDirectory() {
        var loggingDirectoryPath = System.getProperty("user.home") + File.separator + BackendConstants.POPCORN_HOME_DIRECTORY + File.separator;
        var loggingDirectory = new File(loggingDirectoryPath);

        return loggingDirectory.getAbsolutePath();
    }
}
