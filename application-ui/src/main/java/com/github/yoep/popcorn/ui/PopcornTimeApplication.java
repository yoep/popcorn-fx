package com.github.yoep.popcorn.ui;

import com.github.spring.boot.javafx.SpringJavaFXApplication;
import com.github.spring.boot.javafx.stage.BorderlessStageWrapper;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.spring.boot.javafx.view.ViewManager;
import com.github.spring.boot.javafx.view.ViewManagerPolicy;
import com.github.spring.boot.javafx.view.ViewProperties;
import com.github.yoep.popcorn.backend.BackendConstants;
import com.github.yoep.popcorn.backend.FxLibInstance;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.OptionsService;
import com.github.yoep.popcorn.ui.stage.BorderlessStageHolder;
import com.github.yoep.popcorn.ui.view.services.MaximizeService;
import javafx.scene.paint.Color;
import javafx.stage.Stage;
import javafx.stage.StageStyle;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.context.ConfigurableApplicationContext;

import java.awt.*;
import java.io.File;
import java.nio.charset.StandardCharsets;

@Slf4j
@SpringBootApplication
@NoArgsConstructor
public class PopcornTimeApplication extends SpringJavaFXApplication {
    public static final String ICON_NAME = "icon_64.png";
    public static final String APPLICATION_TITLE = "Popcorn Time";

    static final String STAGE_VIEW = "main.fxml";

    public static void main(String[] args) {
        System.setProperty("log.dir", getLogDirectory());
        System.setProperty("jna.encoding", StandardCharsets.UTF_8.name());
        var libArgs = createLibraryArguments(args);
        PopcornFxInstance.INSTANCE.set(FxLibInstance.INSTANCE.get().new_popcorn_fx(libArgs, libArgs.length));
        launch(PopcornTimeApplication.class, PopcornTimePreloader.class, args);
    }

    protected PopcornTimeApplication(ConfigurableApplicationContext applicationContext) {
        this.applicationContext = applicationContext;
    }

    @Override
    public void start(Stage stage) throws Exception {
        log.trace("Starting the application");
        updateStageType(stage);
        super.start(stage);

        var loader = applicationContext.getBean(ViewLoader.class);
        var viewManager = applicationContext.getBean(ViewManager.class);

        log.trace("Loading the main view of the application");
        centerOnActiveScreen(stage);
        loader.show(stage, STAGE_VIEW, getViewProperties());
        viewManager.setPolicy(ViewManagerPolicy.CLOSEABLE);
    }

    private void updateStageType(Stage stage) {
        var settingsService = applicationContext.getBean(ApplicationConfig.class);
        var uiSettings = settingsService.getSettings().getUiSettings();

        if (uiSettings.isNativeWindowEnabled()) {
            log.debug("Showing application in window mode");
        } else {
            log.debug("Showing application in borderless window mode");
            BorderlessStageHolder.setWrapper(new BorderlessStageWrapper(stage));
            stage.initStyle(StageStyle.UNDECORATED);
        }
    }

    private ViewProperties getViewProperties() {
        log.trace("Building the view properties of the application");
        var optionsService = applicationContext.getBean(OptionsService.class);
        var maximizeService = applicationContext.getBean(MaximizeService.class);
        var platformProvider = applicationContext.getBean(PlatformProvider.class);
        var options = optionsService.options();
        var properties = ViewProperties.builder()
                .title(APPLICATION_TITLE)
                .icon(ICON_NAME)
                .background(getBackgroundColor(platformProvider))
                .centerOnScreen(false);

        // check if the big-picture or kiosk mode or maximized is enabled
        // if so, force the application to be maximized
        if (optionsService.isTvMode() || options.isKioskMode() || optionsService.isMaximized()) {
            maximizeService.setMaximized(true);
        } else {
            var settingsService = applicationContext.getBean(ApplicationConfig.class);
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

    private Color getBackgroundColor(PlatformProvider platformProvider) {
        return platformProvider.isTransparentWindowSupported() ?
                Color.TRANSPARENT : Color.BLACK;
    }

    private static void centerOnActiveScreen(Stage stage) {
        var mouse = MouseInfo.getPointerInfo().getLocation();

        stage.setX(mouse.getX());
        stage.setY(mouse.getY());
        stage.centerOnScreen();
    }

    public static String getLogDirectory() {
        var loggingDirectoryPath = System.getProperty("user.home") + File.separator + BackendConstants.POPCORN_HOME_DIRECTORY + File.separator;
        var loggingDirectory = new File(loggingDirectoryPath);

        return loggingDirectory.getAbsolutePath();
    }

    private static String[] createLibraryArguments(String[] args) {
        var libArgs = new String[args.length + 1];
        libArgs[0] = "popcorn-fx";
        System.arraycopy(args, 0, libArgs, 1, args.length);
        return libArgs;
    }
}
