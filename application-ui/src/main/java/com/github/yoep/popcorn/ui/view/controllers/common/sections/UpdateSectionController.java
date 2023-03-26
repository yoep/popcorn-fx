package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.updater.Changelog;
import com.github.yoep.popcorn.backend.updater.UpdateState;
import com.github.yoep.popcorn.backend.updater.VersionInfo;
import com.github.yoep.popcorn.ui.events.CloseUpdateEvent;
import com.github.yoep.popcorn.ui.messages.UpdateMessage;
import com.github.yoep.popcorn.ui.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.ui.view.listeners.UpdateListener;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.popcorn.ui.view.services.UpdateSectionService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import javafx.scene.image.ImageView;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.GridPane;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.stream.Collectors;

@Slf4j
@ViewController
@RequiredArgsConstructor
public class UpdateSectionController implements Initializable {
    private static final String PROGRESS_ERROR_STYLE_CLASS = "error";

    private final UpdateSectionService updateSectionService;
    private final ImageService imageService;
    private final LocaleText localeText;
    private final EventPublisher eventPublisher;

    @FXML
    GridPane updatePane;
    @FXML
    BackgroundImageCover backgroundCover;
    @FXML
    ImageView logoImage;
    @FXML
    Button updateNowButton;
    @FXML
    Label versionLabel;
    @FXML
    Label changelogFeaturesLabel;
    @FXML
    Label changelogBugfixesLabel;
    @FXML
    Label progressLabel;
    @FXML
    ProgressBar progressBarUpdate;
    @FXML
    Pane changelogPane;
    @FXML
    Pane progressPane;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeLogo();
        initializeBackgroundCover();
        initializeListener();
    }

    private void initializeListener() {
        updateSectionService.addListener(new UpdateListener() {
            @Override
            public void onUpdateInfoChanged(VersionInfo newValue) {
                UpdateSectionController.this.onUpdateInfoChanged(newValue);
            }

            @Override
            public void onUpdateStateChanged(UpdateState newState) {
                UpdateSectionController.this.onUpdateStateChanged(newState);
            }
        });
        updatePane.sceneProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue != null) {
                updateNowButton.requestFocus();
            }
        });

        updateSectionService.updateAll();
    }

    private void initializeLogo() {
        imageService.loadResource("icon.png")
                .thenAccept(e -> logoImage.setImage(e));
    }

    private void initializeBackgroundCover() {
        imageService.loadResource("placeholder-background.jpg")
                .thenAccept(e -> backgroundCover.setBackgroundImage(e));
    }

    private void onUpdateInfoChanged(VersionInfo versionInfo) {
        if (versionInfo == null) {
            return;
        }

        Platform.runLater(() -> {
            versionLabel.setText(versionInfo.getVersion());
            changelogFeaturesLabel.setText(Optional.ofNullable(versionInfo.getChangelog())
                    .map(Changelog::getFeatures)
                    .map(e -> e.stream()
                            .map(x -> "- " + x)
                            .collect(Collectors.joining("\n")))
                    .orElse(null));
            changelogBugfixesLabel.setText(Optional.ofNullable(versionInfo.getChangelog())
                    .map(Changelog::getBugfixes)
                    .map(e -> e.stream()
                            .map(x -> "- " + x)
                            .collect(Collectors.joining("\n")))
                    .orElse(null));
        });
    }

    private void onUpdateStateChanged(UpdateState newState) {
        switchPane(newState != UpdateState.UPDATE_AVAILABLE);

        Platform.runLater(() -> {
            switch (newState) {
                case DOWNLOADING -> handleStateChanged(UpdateMessage.DOWNLOADING);
                case DOWNLOAD_FINISHED -> handleStateChanged(UpdateMessage.DOWNLOAD_FINISHED);
                case INSTALLING -> handleStateChanged(UpdateMessage.INSTALLING);
                case ERROR -> handleUpdateErrorState();
            }
        });
    }

    private void handleStateChanged(UpdateMessage message) {
        progressLabel.setText(localeText.get(message));
    }

    private void handleUpdateErrorState() {
        handleStateChanged(UpdateMessage.ERROR);
        progressBarUpdate.setProgress(1.0);
        progressBarUpdate.getStyleClass().add(PROGRESS_ERROR_STYLE_CLASS);
    }

    private void switchPane(boolean isUpdateProgressOngoing) {
        Platform.runLater(() -> {
            changelogPane.setVisible(!isUpdateProgressOngoing);
            progressPane.setVisible(isUpdateProgressOngoing);
        });
    }

    @FXML
    void onUpdateNowClicked(MouseEvent event) {
        event.consume();
        updateSectionService.startUpdate();
    }

    @FXML
    void onUpdateNowPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            updateSectionService.startUpdate();
        }
    }

    @FXML
    void onUpdatePressed(KeyEvent event) {
        if (event.getCode() == KeyCode.BACK_SPACE || event.getCode() == KeyCode.ESCAPE) {
            event.consume();
            eventPublisher.publish(new CloseUpdateEvent(this));
        }
    }
}
