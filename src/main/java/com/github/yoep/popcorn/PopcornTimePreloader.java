package com.github.yoep.popcorn;

import com.github.spring.boot.javafx.view.ViewLoader;
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
    private static final String ICON_NAME = "icon.png";
    private Stage stage;

    @Override
    public void start(Stage primaryStage) throws Exception {
        Image icon = new Image(getIconResource().getInputStream());
        Parent parent = new FXMLLoader(getPreloaderResource().getURL()).load();
        Scene scene = new Scene(parent);

        this.stage = primaryStage;

        primaryStage.setScene(scene);
        primaryStage.setIconified(false);
        primaryStage.initStyle(StageStyle.TRANSPARENT);
        primaryStage.getIcons().add(icon);
        scene.setFill(Color.TRANSPARENT);
        primaryStage.show();
    }

    @Override
    public void handleStateChangeNotification(StateChangeNotification info) {
        if (info.getType() == StateChangeNotification.Type.BEFORE_START) {
            this.stage.close();
        }
    }

    private ClassPathResource getIconResource() {
        return new ClassPathResource(ViewLoader.IMAGE_DIRECTORY + File.separator + ICON_NAME);
    }

    private ClassPathResource getPreloaderResource() {
        return new ClassPathResource(ViewLoader.VIEW_DIRECTORY + "/preloader.fxml");
    }
}
