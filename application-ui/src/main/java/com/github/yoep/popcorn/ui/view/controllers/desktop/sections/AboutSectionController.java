package com.github.yoep.popcorn.ui.view.controllers.desktop.sections;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.info.ComponentInfo;
import com.github.yoep.popcorn.backend.updater.UpdateCallbackEvent;
import com.github.yoep.popcorn.backend.updater.UpdateService;
import com.github.yoep.popcorn.backend.updater.UpdateState;
import com.github.yoep.popcorn.ui.events.CloseAboutEvent;
import com.github.yoep.popcorn.ui.events.ShowAboutEvent;
import com.github.yoep.popcorn.ui.messages.UpdateMessage;
import com.github.yoep.popcorn.ui.view.controls.AboutDetails;
import com.github.yoep.popcorn.ui.view.controls.ImageCover;
import com.github.yoep.popcorn.ui.view.listeners.AboutSectionListener;
import com.github.yoep.popcorn.ui.view.services.AboutSectionService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.image.ImageView;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;

@Slf4j
@ViewController
@RequiredArgsConstructor
public class AboutSectionController implements Initializable {
    private final AboutSectionService aboutService;
    private final ImageService imageService;
    private final EventPublisher eventPublisher;
    private final UpdateService updateService;
    private final LocaleText localeText;
    private final FxLib fxLib;

    @FXML
    ImageCover backgroundCover;
    @FXML
    ImageView logoImage;
    @FXML
    Label versionLabel;
    @FXML
    AboutDetails playersPane;
    @FXML
    AboutDetails videoPane;
    @FXML
    Button updateButton;
    @FXML
    Icon updateIcon;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeLogo();
        initializeBackgroundCover();
        initializeLabels();
        initializeListeners();
        initializeButton();
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
        eventPublisher.register(ShowAboutEvent.class, event -> {
            Platform.runLater(() -> updateButton.requestFocus());
            return event;
        });
    }

    private void initializeButton() {
        updateService.register(event -> {
            if (event.getTag() == UpdateCallbackEvent.Tag.StateChanged) {
                onUpdateStateChanged(event.getUnion().getState_changed().getNewState());
            }
        });
        onUpdateStateChanged(updateService.getState());
    }

    private void onUpdateStateChanged(UpdateState newState) {
        Platform.runLater(() -> {
            switch (newState) {
                case UPDATE_AVAILABLE -> {
                    updateButton.setText(localeText.get(UpdateMessage.DOWNLOAD_UPDATE));
                    updateIcon.setText(Icon.DOWNLOAD_UNICODE);
                }
                case NO_UPDATE_AVAILABLE -> {
                    updateButton.setText(localeText.get(UpdateMessage.CHECK_FOR_NEW_UPDATES));
                    updateIcon.setText(Icon.REFRESH_UNICODE);
                }
                case ERROR -> {
                    updateButton.setText(localeText.get(UpdateMessage.NO_UPDATE_AVAILABLE));
                    updateIcon.setText(Icon.TIMES_UNICODE);
                }
            }
        });
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
