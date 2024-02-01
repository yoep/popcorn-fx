package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.loader.*;
import com.github.yoep.popcorn.ui.events.CloseLoadEvent;
import com.github.yoep.popcorn.ui.messages.TorrentMessage;
import com.github.yoep.popcorn.ui.utils.ProgressUtils;
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
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@ViewController
@RequiredArgsConstructor
public class LoaderTorrentComponent implements Initializable {
    static final String PROGRESS_ERROR_STYLE_CLASS = "error";

    private final LocaleText localeText;
    private final ImageService imageService;
    private final LoaderService loaderService;
    private final EventPublisher eventPublisher;

    @FXML
    Pane loaderActions;
    @FXML
    Label progressPercentage;
    @FXML
    Label downloadText;
    @FXML
    Label uploadText;
    @FXML
    Label activePeersText;
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

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeProgressBar();
        initializeRetryButton();
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

    @PostConstruct
    void init() {
        loaderService.addListener(new LoaderListener() {
            @Override
            public void onLoadingStarted(LoadingStartedEventC loadingStartedEvent) {
                Platform.runLater(() -> {
                    onLoadingInitializing();
                    backgroundImage.reset();
                    loadingStartedEvent.getBackground()
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
            public void onStateChanged(LoaderState newState) {
                Platform.runLater(() -> handleLoaderStateChanged(newState));
            }

            @Override
            public void onProgressChanged(LoadingProgress progress) {
                Platform.runLater(() -> onLoadingProgressChanged(progress));
            }

            @Override
            public void onError(LoadingErrorC error) {
                Platform.runLater(() -> onLoadTorrentError());
            }
        });
    }

    //endregion

    //region Functions

    private void handleLoaderStateChanged(LoaderState newState) {
        switch (newState) {
            case IDLE, INITIALIZING -> onLoadingInitializing();
            case STARTING -> {
                reset();
                progressStatus.setVisible(false);
                statusText.setText(localeText.get(TorrentMessage.STARTING));
            }
            case RETRIEVING_SUBTITLES -> statusText.setText(localeText.get(TorrentMessage.RETRIEVING_SUBTITLES));
            case DOWNLOADING_SUBTITLE -> statusText.setText(localeText.get(TorrentMessage.DOWNLOADING_SUBTITLE));
            case CONNECTING -> statusText.setText(localeText.get(TorrentMessage.CONNECTING));
            case DOWNLOADING -> {
                progressStatus.setVisible(true);
                statusText.setText(localeText.get(TorrentMessage.DOWNLOADING));
            }
            case DOWNLOAD_FINISHED, READY -> {
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

    private void onLoadingProgressChanged(LoadingProgress progress) {
        progressStatus.setVisible(true);
        progressBar.setProgress(progress.getProgress());
        progressBar.setVisible(true);
        statusText.setText(localeText.get(TorrentMessage.DOWNLOADING));
        progressPercentage.setText(ProgressUtils.progressToPercentage(progress));
        downloadText.setText(ProgressUtils.progressToDownload(progress));
        uploadText.setText(ProgressUtils.progressToUpload(progress));
        activePeersText.setText(String.valueOf(progress.getSeeds()));
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
