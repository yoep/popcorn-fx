package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.ui.view.services.MaximizeService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.input.MouseEvent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.ClassPathResource;

import javax.annotation.PostConstruct;
import java.io.IOException;
import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class WindowComponent implements Initializable {
    private final MaximizeService maximizeService;
    private final PlatformProvider platformProvider;

    private Image restoreImage;
    private Image maximizeImage;

    @FXML
    ImageView maximizeImageView;

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeMaximizeImage();
    }

    private void initializeMaximizeImage() {
        onMaximizedStateChanged(maximizeService.isMaximized());
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        initializeListeners();
        initializeImages();
    }

    private void initializeListeners() {
        maximizeService.maximizedProperty().addListener((observable, oldValue, newValue) -> onMaximizedStateChanged(newValue));
    }

    private void initializeImages() {
        var restore = new ClassPathResource("/images/windows/restore.png");
        var maximize = new ClassPathResource("/images/windows/maximize.png");

        try {
            restoreImage = new Image(restore.getInputStream());
            maximizeImage = new Image(maximize.getInputStream());
        } catch (IOException ex) {
            log.error("Failed to initialize title bar images, " + ex.getMessage(), ex);
        }
    }

    //endregion

    //region Functions

    private void onMaximizedStateChanged(boolean maximized) {
        if (maximized) {
            maximizeImageView.setImage(restoreImage);
        } else {
            maximizeImageView.setImage(maximizeImage);
        }
    }

    private void switchMaximizedState() {
        maximizeService.setMaximized(!maximizeService.isMaximized());
    }

    @FXML
    void onMinimizeClicked(MouseEvent event) {
        event.consume();
        maximizeService.minimize();
    }

    @FXML
    void onMaximizedClicked(MouseEvent event) {
        event.consume();
        switchMaximizedState();
    }

    @FXML
    void onCloseClicked(MouseEvent event) {
        event.consume();
        platformProvider.exit();
    }

    //endregion
}
