package com.github.yoep.popcorn.ui.view.controllers.desktop.sections;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.info.ComponentInfo;
import com.github.yoep.popcorn.ui.events.CloseAboutEvent;
import com.github.yoep.popcorn.ui.view.controls.AboutDetails;
import com.github.yoep.popcorn.ui.view.controls.ImageCover;
import com.github.yoep.popcorn.ui.view.listeners.AboutSectionListener;
import com.github.yoep.popcorn.ui.view.services.AboutSectionService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.image.ImageView;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationContext;

import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;

@Slf4j
@ViewController
@RequiredArgsConstructor
public class AboutSectionController implements Initializable {
    private final ApplicationContext applicationContext;
    private final AboutSectionService aboutService;
    private final ImageService imageService;
    private final EventPublisher eventPublisher;
    private final FxLib fxLib;

    @FXML
    ImageCover backgroundCover;
    @FXML
    ImageView logoImage;
    @FXML
    Label titleLabel;
    @FXML
    Label versionLabel;
    @FXML
    AboutDetails playersPane;
    @FXML
    AboutDetails videoPane;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeLogo();
        initializeBackgroundCover();
        initializeLabels();
        initializeListeners();
    }

    private void initializeLogo() {
        imageService.loadResource("icon.png")
                .thenAccept(e -> logoImage.setImage(e));
    }

    private void initializeBackgroundCover() {
        imageService.loadResource("bg-header.jpg")
                .thenAccept(e -> backgroundCover.setImage(e));
    }

    private void initializeLabels() {
        titleLabel.setText(applicationContext.getId());
        versionLabel.setText(fxLib.version());
    }

    private void initializeListeners() {
        aboutService.addListener(new AboutSectionListener() {
            @Override
            public void onPlayersChanged(List<ComponentInfo> players) {
                AboutSectionController.this.onPlayersChanged(players);
            }

            @Override
            public void onVideoPlayersChanged(List<ComponentInfo> videoPlayers) {
                AboutSectionController.this.onVideoPlayersChanged(videoPlayers);
            }
        });
        aboutService.updateAll();
    }

    private void onPlayersChanged(List<ComponentInfo> players) {
        Platform.runLater(() -> playersPane.setItems(players));
    }

    private void onVideoPlayersChanged(List<ComponentInfo> videoPlayers) {
        Platform.runLater(() -> videoPane.setItems(videoPlayers));
    }

    @FXML
    void onAboutPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.BACK_SPACE || event.getCode() == KeyCode.ESCAPE) {
            event.consume();
            eventPublisher.publish(new CloseAboutEvent(this));
        }
    }
}
