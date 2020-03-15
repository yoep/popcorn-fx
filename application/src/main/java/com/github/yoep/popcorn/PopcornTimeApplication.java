package com.github.yoep.popcorn;

import com.github.spring.boot.javafx.SpringJavaFXApplication;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.spring.boot.javafx.view.ViewManager;
import com.github.spring.boot.javafx.view.ViewManagerPolicy;
import com.github.spring.boot.javafx.view.ViewProperties;
import com.github.yoep.popcorn.settings.OptionsService;
import com.github.yoep.popcorn.settings.SettingsService;
import javafx.application.ConditionalFeature;
import javafx.application.Platform;
import javafx.scene.paint.Color;
import javafx.stage.Stage;
import javafx.stage.StageStyle;
import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.boot.autoconfigure.security.servlet.SecurityAutoConfiguration;

import java.io.File;

@Slf4j
@SpringBootApplication(exclude = {
        SecurityAutoConfiguration.class
})
public class PopcornTimeApplication extends SpringJavaFXApplication {
    public static final String ICON_NAME = "icon_64.png";
    public static final String APP_DIR = getDefaultAppDirLocation();

    public static void main(String[] args) {
        launch(PopcornTimeApplication.class, PopcornTimePreloader.class, args);
    }

    @Override
    public void start(Stage primaryStage) throws Exception {
        log.trace("Starting the application");
        super.start(primaryStage);
        var loader = applicationContext.getBean(ViewLoader.class);
        var viewManager = applicationContext.getBean(ViewManager.class);

//        initializeStage(primaryStage);

        log.trace("Loading the main view of the application");
        loader.show(primaryStage, "main.fxml", getViewProperties());
        viewManager.setPolicy(ViewManagerPolicy.CLOSEABLE);
    }

    private void initializeStage(Stage primaryStage) {
        if (Platform.isSupported(ConditionalFeature.TRANSPARENT_WINDOW)) {
            primaryStage.initStyle(StageStyle.TRANSPARENT);
        } else {
            primaryStage.initStyle(StageStyle.UNDECORATED);
        }
    }

    private ViewProperties getViewProperties() {
        log.trace("Building the view properties of the application");
        var optionsService = applicationContext.getBean(OptionsService.class);
        var options = optionsService.options();
        var properties = ViewProperties.builder()
                .title("Popcorn Time")
                .icon(ICON_NAME)
                .background(Color.BLACK);

        // check if the big-picture or kiosk mode is enabled
        // if so, force the application to be maximized
        if (options.isBigPictureMode() || options.isKioskMode()) {
            properties.maximized(true);
        } else {
            var settingsService = applicationContext.getBean(SettingsService.class);
            var uiSettings = settingsService.getSettings().getUiSettings();

            properties.maximized(uiSettings.isMaximized());
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

    private static String getDefaultAppDirLocation() {
        return System.getProperty("user.home") + File.separator + ".popcorn-time" + File.separator;
    }
}
