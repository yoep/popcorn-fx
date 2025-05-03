package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Update;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.UpdateEvent;
import com.github.yoep.popcorn.backend.messages.UpdateMessage;
import com.github.yoep.popcorn.backend.updater.UpdateEventListener;
import com.github.yoep.popcorn.backend.updater.UpdateService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.CloseUpdateEvent;
import com.github.yoep.popcorn.ui.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import javafx.scene.control.ProgressIndicator;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.GridPane;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class UpdateSectionController implements Initializable {
    private static final String PROGRESS_ERROR_STYLE_CLASS = "error";

    private final UpdateService updateService;
    private final ImageService imageService;
    private final LocaleText localeText;
    private final EventPublisher eventPublisher;

    @FXML
    GridPane updatePane;
    @FXML
    BackgroundImageCover backgroundCover;
    @FXML
    Label progressLabel;
    @FXML
    ProgressBar progressBar;
    @FXML
    Pane progressPane;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeBackgroundCover();
        initializeListener();
    }

    private void initializeListener() {
        updateService.addListener(new UpdateEventListener() {
            @Override
            public void onStateChanged(UpdateEvent.StateChanged event) {
                onUpdateStateChanged(event.getNewState());
            }

            @Override
            public void onDownloadProgress(UpdateEvent.DownloadProgress event) {
                onUpdateDownloadProgress(event.getProgress());
            }
        });
        updatePane.sceneProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue != null) {
                updateService.startUpdateDownload();
            }
        });
        updateService.getState().whenComplete((state, throwable) -> {
            if (throwable == null) {
                onUpdateStateChanged(state);
            } else {
                log.error("Failed to retrieve update state", throwable);
            }
        });
    }

    private void initializeBackgroundCover() {
        imageService.loadResource("placeholder-background.jpg")
                .thenAccept(e -> backgroundCover.setBackgroundImage(e));
    }

    private void onUpdateStateChanged(Update.State newState) {
        Platform.runLater(() -> {
            switch (newState) {
                case DOWNLOADING -> handleStateChanged(UpdateMessage.STARTING_DOWNLOAD);
                case DOWNLOAD_FINISHED -> {
                    handleStateChanged(UpdateMessage.DOWNLOAD_FINISHED);
                    updateService.startUpdateInstallation();
                }
                case INSTALLING -> {
                    handleStateChanged(UpdateMessage.INSTALLING);
                    Platform.runLater(() -> progressBar.setProgress(ProgressIndicator.INDETERMINATE_PROGRESS));
                }
                case ERROR -> handleUpdateErrorState();
            }
        });
    }

    private void onUpdateDownloadProgress(Update.DownloadProgress downloadProgress) {
        var progress = ((double) downloadProgress.getDownloaded()) / downloadProgress.getTotalSize();
        var percentage = (int) (progress * 100);

        Platform.runLater(() -> {
            progressBar.setProgress(progress);
            progressLabel.setText(localeText.get(UpdateMessage.DOWNLOADING, percentage));
        });
    }

    private void handleStateChanged(UpdateMessage message) {
        progressBar.getStyleClass().removeIf(e -> e.equals(PROGRESS_ERROR_STYLE_CLASS));
        progressLabel.setText(localeText.get(message));
    }

    private void handleUpdateErrorState() {
        handleStateChanged(UpdateMessage.ERROR);
        progressBar.setProgress(1.0);
        progressBar.getStyleClass().add(PROGRESS_ERROR_STYLE_CLASS);
    }

    @FXML
    void onUpdatePressed(KeyEvent event) {
        if (event.getCode() == KeyCode.BACK_SPACE || event.getCode() == KeyCode.ESCAPE) {
            event.consume();
            eventPublisher.publish(new CloseUpdateEvent(this));
        }
    }
}
