package com.github.yoep.popcorn.ui;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.FxLibInstance;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import javafx.application.ConditionalFeature;
import javafx.application.Platform;
import javafx.application.Preloader;
import javafx.fxml.FXMLLoader;
import javafx.scene.Cursor;
import javafx.scene.Parent;
import javafx.scene.Scene;
import javafx.scene.control.Alert;
import javafx.scene.image.Image;
import javafx.scene.paint.Color;
import javafx.stage.Stage;
import javafx.stage.StageStyle;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.ClassPathResource;

import java.awt.*;
import java.io.File;

@Slf4j
public class PopcornTimePreloader extends Preloader {
    private static final String OPTIONS_PREFIX = "--";

    private Stage stage;

    @Override
    public void start(Stage primaryStage) throws Exception {
        var icon = new Image(getIconResource().getInputStream());
        var parent = new FXMLLoader(getPreloaderResource().getURL()).<Parent>load();
        var scene = new Scene(parent);
        var mouse = MouseInfo.getPointerInfo().getLocation();

        this.stage = primaryStage;

        primaryStage.setTitle(PopcornTimeApplication.APPLICATION_TITLE);
        primaryStage.setScene(scene);
        primaryStage.setIconified(false);
        primaryStage.getIcons().add(icon);
        primaryStage.setX(mouse.getX());
        primaryStage.setY(mouse.getY());

        updateBackground(primaryStage, scene);
        processParameters(primaryStage, scene);

        primaryStage.show();
        primaryStage.centerOnScreen();
    }

    @Override
    public void handleStateChangeNotification(StateChangeNotification info) {
        log.trace("Received state change notification {}", info.getType());
        if (info.getType() == StateChangeNotification.Type.BEFORE_START) {
            log.trace("Closing preloader stage");
            this.stage.close();
            this.stage = null;
        }
    }

    @Override
    public boolean handleErrorNotification(ErrorNotification info) {
        log.error("Application failed to load, " + info.getDetails(), info.getCause());
        var alert = new Alert(Alert.AlertType.ERROR, "Application failed to start, check logs for more information");

        alert.showAndWait();

        // close the JVM with a failure indication
        Platform.exit();
        System.exit(1);

        return true;
    }

    private void updateBackground(Stage stage, Scene scene) {
        if (isTransparencySupported()) {
            stage.initStyle(StageStyle.TRANSPARENT);
            scene.setFill(Color.TRANSPARENT);
        } else {
            log.debug("Transparency is not supported, using black preloader background instead");
            stage.initStyle(StageStyle.UNDECORATED);
            scene.setFill(Color.BLACK);
        }
    }

    private void processParameters(Stage primaryStage, Scene scene) {
        // check if the mouse should be hidden
        if (FxLibInstance.INSTANCE.get().is_mouse_disabled(PopcornFxInstance.INSTANCE.get()) == (byte) 1) {
            log.trace("Hiding preloader cursor");
            scene.setCursor(Cursor.NONE);
        }
    }

    private ClassPathResource getIconResource() {
        return new ClassPathResource(ViewLoader.IMAGE_DIRECTORY + File.separator + PopcornTimeApplication.ICON_NAME);
    }

    private ClassPathResource getPreloaderResource() {
        return new ClassPathResource(ViewLoader.VIEW_DIRECTORY + "/preloader.fxml");
    }

    /**
     * Check if transparency is supported by the current system.
     *
     * @return Returns true if the system supports transparent windows, else false.
     */
    private static boolean isTransparencySupported() {
        return Platform.isSupported(ConditionalFeature.TRANSPARENT_WINDOW);
    }
}
