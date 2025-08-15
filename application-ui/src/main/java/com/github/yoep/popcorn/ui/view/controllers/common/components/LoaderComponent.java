package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.LoaderEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Loading;
import com.github.yoep.popcorn.backend.loader.*;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.CloseLoadEvent;
import com.github.yoep.popcorn.ui.messages.TorrentMessage;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import com.github.yoep.popcorn.ui.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Optional;
import java.util.ResourceBundle;

@Slf4j
public class LoaderComponent implements Initializable {
    static final String PROGRESS_ERROR_STYLE_CLASS = "error";
    static final String PROGRESS_INFO_VIEW = "common/components/progress-info.component.fxml";

    private final LocaleText localeText;
    private final ImageService imageService;
    private final LoaderService loaderService;
    private final EventPublisher eventPublisher;
    private final ViewLoader viewLoader;
    final ProgressInfoComponent infoComponent = new ProgressInfoComponent();

    @FXML
    Pane loaderActions;
    @FXML
    Pane infoPane;
    @FXML
    Pane progressStatus;
    @FXML
    BackgroundImageCover backgroundImage;
    @FXML
    Label statusText;
    @FXML
    ProgressBar progressBar;
    @FXML
    Button loadRetryButton;

    public LoaderComponent(LocaleText localeText, ImageService imageService, LoaderService loaderService, EventPublisher eventPublisher, ViewLoader viewLoader) {
        this.localeText = localeText;
        this.imageService = imageService;
        this.loaderService = loaderService;
        this.eventPublisher = eventPublisher;
        this.viewLoader = viewLoader;
        init();
    }

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeProgressInfo();
        initializeProgressBar();
        initializeRetryButton();
    }

    private void initializeProgressInfo() {
        infoPane.getChildren().remove(progressStatus);
        progressStatus = viewLoader.load(PROGRESS_INFO_VIEW, infoComponent);
        infoPane.getChildren().add(2, progressStatus);
    }

    private void initializeProgressBar() {
        progressBar.setProgress(0.0);
        progressBar.setVisible(false);
    }

    private void initializeRetryButton() {
        removeRetryButton();
    }

    //endregion

    //region Init

    private void init() {
        loaderService.addListener(new LoaderListener() {
            @Override
            public void onLoadingStarted(LoaderEvent.LoadingStarted loadingStartedEvent) {
                Platform.runLater(() -> {
                    onLoadingInitializing();
                    backgroundImage.reset();
                    Optional.ofNullable(loadingStartedEvent.getBackground())
                            .filter(e -> loadingStartedEvent.hasBackground())
                            .map(imageService::load)
                            .ifPresent(e -> e.whenComplete((bytes, throwable) -> {
                                if (throwable == null) {
                                    Platform.runLater(() -> backgroundImage.setBackgroundImage(bytes));
                                } else {
                                    log.error(throwable.getMessage(), throwable);
                                }
                            }));
                });
            }

            @Override
            public void onStateChanged(Loading.State newState) {
                Platform.runLater(() -> handleLoaderStateChanged(newState));
            }

            @Override
            public void onProgressChanged(Loading.Progress progress) {
                Platform.runLater(() -> onLoadingProgressChanged(progress));
            }

            @Override
            public void onError(Loading.Error error) {
                Platform.runLater(() -> onLoadTorrentError());
            }
        });
    }

    //endregion

    //region Functions

    private void handleLoaderStateChanged(Loading.State newState) {
        if (newState == null)
            return;

        switch (newState) {
            case INITIALIZING -> onLoadingInitializing();
            case STARTING -> {
                reset();
                progressStatus.setVisible(false);
                statusText.setText(localeText.get(TorrentMessage.STARTING));
            }
            case RETRIEVING_SUBTITLES -> statusText.setText(localeText.get(TorrentMessage.RETRIEVING_SUBTITLES));
            case DOWNLOADING_SUBTITLE -> statusText.setText(localeText.get(TorrentMessage.DOWNLOADING_SUBTITLE));
            case RETRIEVING_METADATA -> statusText.setText(localeText.get(TorrentMessage.RETRIEVING_METADATA));
            case VERIFYING_FILES -> statusText.setText(localeText.get(TorrentMessage.VERIFYING_FILES));
            case CONNECTING -> statusText.setText(localeText.get(TorrentMessage.CONNECTING));
            case DOWNLOADING -> {
                progressStatus.setVisible(true);
                statusText.setText(localeText.get(TorrentMessage.DOWNLOADING));
            }
            case READY -> {
                statusText.setText(localeText.get(TorrentMessage.READY));
                progressBar.setProgress(1);
                progressBar.setVisible(true);
            }
        }
    }

    private void onLoadingInitializing() {
        reset();
        progressStatus.setVisible(false);
    }

    private void onLoadingProgressChanged(Loading.Progress progress) {
        progressStatus.setVisible(true);
        progressBar.setProgress(progress.getProgress());
        progressBar.setVisible(true);
        statusText.setText(localeText.get(TorrentMessage.DOWNLOADING));
        infoComponent.update(new DownloadStatus() {
            @Override
            public float progress() {
                return Float.min(progress.getProgress(), 1);
            }

            @Override
            public int seeds() {
                return progress.getSeeds();
            }

            @Override
            public int peers() {
                return progress.getPeers();
            }

            @Override
            public int downloadSpeed() {
                return (int) progress.getDownloadSpeed();
            }

            @Override
            public int uploadSpeed() {
                return (int) progress.getUploadSpeed();
            }

            @Override
            public long downloaded() {
                return progress.getDownloaded();
            }

            @Override
            public long totalSize() {
                return progress.getTotalSize();
            }
        });
    }

    private void onLoadTorrentError() {
        // update the actions with the retry button
        loaderActions.getChildren().add(0, loadRetryButton);

        statusText.setText(localeText.get(TorrentMessage.FAILED));
        progressBar.setProgress(1);
        progressBar.setVisible(true);
        progressBar.getStyleClass().add(PROGRESS_ERROR_STYLE_CLASS);
    }

    private void reset() {
        Platform.runLater(() -> {
            statusText.setText(localeText.get(TorrentMessage.INITIALIZING));
            progressBar.getStyleClass().removeIf(e -> e.equals(PROGRESS_ERROR_STYLE_CLASS));
        });

        resetProgressToDefaultState();
        removeRetryButton();
    }

    private void resetProgressToDefaultState() {
        Platform.runLater(() -> {
            progressBar.setProgress(0.0);
            progressBar.setVisible(false);
            progressBar.getStyleClass().removeIf(e -> e.equals(PROGRESS_ERROR_STYLE_CLASS));
        });
    }

    private void removeRetryButton() {
        Platform.runLater(() -> loaderActions.getChildren().removeIf(e -> e == loadRetryButton));
    }

    private void close() {
        log.debug("Cancelling torrent loader");
        loaderService.cancel();
        reset();
        eventPublisher.publishEvent(new CloseLoadEvent(this));
    }

    @FXML
    void onCancelClicked(MouseEvent event) {
        event.consume();
        close();
    }

    @FXML
    void onRetryClicked(MouseEvent event) {
        event.consume();
        // TODO
    }

    @FXML
    void onCancelPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            close();
        }
    }

    @FXML
    void onLoaderKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.BACK_SPACE || event.getCode() == KeyCode.ESCAPE) {
            event.consume();
            close();
        }
    }

    //endregion
}
