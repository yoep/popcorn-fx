package com.github.yoep.popcorn;

import com.github.spring.boot.javafx.SpringJavaFXApplication;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.spring.boot.javafx.view.ViewManager;
import com.github.spring.boot.javafx.view.ViewManagerPolicy;
import com.github.spring.boot.javafx.view.ViewProperties;
import com.github.yoep.popcorn.settings.SettingsService;
import javafx.scene.paint.Color;
import javafx.stage.Stage;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.boot.autoconfigure.security.servlet.SecurityAutoConfiguration;

import java.io.File;

@SpringBootApplication(exclude = {
        SecurityAutoConfiguration.class
})
public class PopcornTimeApplication extends SpringJavaFXApplication {
    public static final String ICON_NAME = "icon_64.png";
    public static String APP_DIR = getDefaultAppDirLocation();

    public static void main(String[] args) {
        launch(PopcornTimeApplication.class, PopcornTimePreloader.class, args);
    }

    @Override
    public void start(Stage primaryStage) throws Exception {
        super.start(primaryStage);
        var loader = applicationContext.getBean(ViewLoader.class);
        var viewManager = applicationContext.getBean(ViewManager.class);
        var settingsService = applicationContext.getBean(SettingsService.class);
        var uiSettings = settingsService.getSettings().getUiSettings();

        loader.show(primaryStage, "main.fxml", ViewProperties.builder()
                .title("Popcorn Time")
                .icon(ICON_NAME)
                .maximized(uiSettings.isMaximized())
                .background(Color.BLACK)
                .build());
        viewManager.setPolicy(ViewManagerPolicy.CLOSEABLE);
    }

    private static String getDefaultAppDirLocation() {
        return System.getProperty("user.home") + File.separator + ".popcorn-time" + File.separator;
    }
}
