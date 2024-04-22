package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowAboutEvent;
import com.github.yoep.popcorn.backend.info.ComponentInfo;
import com.github.yoep.popcorn.backend.messages.UpdateMessage;
import com.github.yoep.popcorn.backend.updater.UpdateCallbackEvent;
import com.github.yoep.popcorn.backend.updater.UpdateService;
import com.github.yoep.popcorn.backend.updater.UpdateState;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.CloseAboutEvent;
import com.github.yoep.popcorn.ui.events.ShowUpdateEvent;
import com.github.yoep.popcorn.ui.font.controls.Icon;
import com.github.yoep.popcorn.ui.view.controls.AboutDetails;
import com.github.yoep.popcorn.ui.view.controls.ImageCover;
import com.github.yoep.popcorn.ui.view.listeners.AboutSectionListener;
import com.github.yoep.popcorn.ui.view.services.AboutSectionService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.animation.Animation;
import javafx.animation.RotateTransition;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.image.ImageView;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.util.Duration;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;

@Slf4j

@RequiredArgsConstructor
public class AboutSectionController implements Initializable {
    private final AboutSectionService aboutService;
    private final ImageService imageService;
    private final EventPublisher eventPublisher;
    private final UpdateService updateService;
    private final LocaleText localeText;
    private final FxLib fxLib;

    private final RotateTransition updateAnimation = new RotateTransition(Duration.seconds(1));

    @FXML
    ImageCover backgroundCover;
    @FXML
    ImageView logoImage;
    @FXML
    Label versionLabel;
    @FXML
    Label newVersionLabel;
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
        updateAnimation.setNode(updateButton.getGraphic());
        updateAnimation.setCycleCount(Animation.INDEFINITE);
        updateAnimation.setFromAngle(0.0);
        updateAnimation.setToAngle(360.0);
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
                case CHECKING_FOR_NEW_VERSION -> {
                    updateButton.setText(localeText.get(UpdateMessage.CHECKING_FOR_UPDATES));
                    updateIcon.setText(Icon.REFRESH_UNICODE);
                    newVersionLabel.setText(null);
                    updateAnimation.playFromStart();
                }
                case UPDATE_AVAILABLE -> {
                    updateButton.setText(localeText.get(UpdateMessage.DOWNLOAD_UPDATE));
                    updateIcon.setText(Icon.DOWNLOAD_UNICODE);
                    updateService.getUpdateInfo().ifPresent(e -> newVersionLabel.setText(localeText.get(UpdateMessage.NEW_VERSION, e.getApplication().getVersion())));
                    updateAnimation.stop();
                }
                case NO_UPDATE_AVAILABLE -> {
                    updateButton.setText(localeText.get(UpdateMessage.CHECK_FOR_NEW_UPDATES));
                    updateIcon.setText(Icon.REFRESH_UNICODE);
                    newVersionLabel.setText(null);
                    updateAnimation.stop();
                }
                case ERROR -> {
                    updateButton.setText(localeText.get(UpdateMessage.NO_UPDATE_AVAILABLE));
                    updateIcon.setText(Icon.TIMES_UNICODE);
                    newVersionLabel.setText(null);
                    updateAnimation.stop();
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

    private void onUpdate() {
        if (updateService.getState() == UpdateState.UPDATE_AVAILABLE) {
            eventPublisher.publish(new ShowUpdateEvent(this));
        } else {
            updateService.checkForUpdates();
        }
    }

    @FXML
    void onAboutPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.BACK_SPACE || event.getCode() == KeyCode.ESCAPE) {
            event.consume();
            eventPublisher.publish(new CloseAboutEvent(this));
        }
    }

    @FXML
    void onUpdateClicked(MouseEvent event) {
        event.consume();
        onUpdate();
    }

    @FXML
    void onUpdatePressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onUpdate();
        }
    }
}
