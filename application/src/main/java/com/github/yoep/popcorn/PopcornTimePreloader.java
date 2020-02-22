package com.github.yoep.popcorn;

import com.github.spring.boot.javafx.view.ViewLoader;
import javafx.application.ConditionalFeature;
import javafx.application.Platform;
import javafx.application.Preloader;
import javafx.fxml.FXMLLoader;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.scene.image.Image;
import javafx.scene.paint.Color;
import javafx.stage.Stage;
import javafx.stage.StageStyle;
import org.springframework.core.io.ClassPathResource;

import java.io.File;

public class PopcornTimePreloader extends Preloader {
    private Stage stage;

    @Override
    public void start(Stage primaryStage) throws Exception {
        var icon = new Image(getIconResource().getInputStream());
        var parent = new FXMLLoader(getPreloaderResource().getURL()).<Parent>load();
        var scene = new Scene(parent);

        this.stage = primaryStage;

        primaryStage.setScene(scene);
        primaryStage.setIconified(false);
        primaryStage.getIcons().add(icon);

        updateBackground(primaryStage, scene);

        primaryStage.show();
    }

    @Override
    public void handleStateChangeNotification(StateChangeNotification info) {
        if (info.getType() == StateChangeNotification.Type.BEFORE_START) {
            this.stage.close();
        }
    }

    private void updateBackground(Stage stage, Scene scene) {
        if (isTransparencySupported()) {
            stage.initStyle(StageStyle.TRANSPARENT);
            scene.setFill(Color.TRANSPARENT);
        } else {
            stage.initStyle(StageStyle.UNDECORATED);
            scene.setFill(Color.BLACK);
        }
    }

    private ClassPathResource getIconResource() {
        return new ClassPathResource(ViewLoader.IMAGE_DIRECTORY + File.separator + PopcornTimeApplication.ICON_NAME);
    }

    private ClassPathResource getPreloaderResource() {
        return new ClassPathResource(ViewLoader.VIEW_DIRECTORY + "/preloader.fxml");
    }

    private static boolean isTransparencySupported() {
        return Platform.isSupported(ConditionalFeature.TRANSPARENT_WINDOW);
    }
}
