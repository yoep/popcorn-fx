package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.settings.OptionsService;
import com.github.yoep.popcorn.ui.view.services.MaximizeService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.ClassPathResource;

import javax.annotation.PostConstruct;
import java.io.IOException;
import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class TitleBarComponent implements Initializable {
    private final MaximizeService maximizeService;
    private final OptionsService optionsService;

    private Image restoreImage;
    private Image maximizeImage;

    @FXML
    private ImageView maximizeImageView;
    @FXML
    private Pane osButtons;

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeMaximizeImage();
        initializeOsButtons();
    }

    private void initializeMaximizeImage() {
        onMaximizedStateChanged(maximizeService.isMaximized());
    }

    private void initializeOsButtons() {
        var options = optionsService.options();

        osButtons.setVisible(!options.isKioskMode());
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
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
    private void onMinimizeClicked(MouseEvent event) {
        event.consume();
        maximizeService.minimize();
    }

    @FXML
    private void onMaximizedClicked(MouseEvent event) {
        event.consume();
        switchMaximizedState();
    }

    @FXML
    private void onCloseClicked(MouseEvent event) {
        event.consume();
        Platform.exit();
    }

    @FXML
    private void onTitleBarClicked(MouseEvent event) {
        if (event.getButton() == MouseButton.PRIMARY && event.getClickCount() == 2) {
            event.consume();
            maximizeService.setMaximized(!maximizeService.isMaximized());
        }
    }

    //endregion
}
