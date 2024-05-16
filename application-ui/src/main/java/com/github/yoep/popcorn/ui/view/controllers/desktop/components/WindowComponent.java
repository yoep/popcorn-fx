package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.ui.view.services.MaximizeService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.input.MouseEvent;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Objects;
import java.util.ResourceBundle;

@Slf4j
public class WindowComponent implements Initializable {
    private final MaximizeService maximizeService;
    private final PlatformProvider platformProvider;

    final Image restoreImage;
    final Image maximizeImage;

    @FXML
    ImageView maximizeImageView;

    public WindowComponent(MaximizeService maximizeService, PlatformProvider platformProvider) {
        Objects.requireNonNull(maximizeService, "maximizeService cannot be null");
        Objects.requireNonNull(platformProvider, "platformProvider cannot be null");
        this.maximizeService = maximizeService;
        this.platformProvider = platformProvider;
        this.restoreImage = new Image(WindowComponent.class.getResourceAsStream("/images/windows/restore.png"));
        this.maximizeImage = new Image(WindowComponent.class.getResourceAsStream("/images/windows/maximize.png"));
        init();
    }

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

    private void init() {
        initializeListeners();
    }

    private void initializeListeners() {
        maximizeService.maximizedProperty().addListener((observable, oldValue, newValue) -> onMaximizedStateChanged(newValue));
    }

    //endregion

    //region Functions

    private void onMaximizedStateChanged(boolean maximized) {
        if (maximizeImageView == null)
            return;

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
        platformProvider.exit(0);
    }

    //endregion
}
