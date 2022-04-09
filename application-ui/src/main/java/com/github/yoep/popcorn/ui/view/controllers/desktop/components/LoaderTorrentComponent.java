package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.ui.messages.TorrentMessage;
import com.github.yoep.popcorn.ui.utils.ProgressUtils;
import com.github.yoep.popcorn.ui.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.ui.view.listeners.LoadTorrentListener;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.popcorn.ui.view.services.LoadTorrentService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
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

    private final LoadTorrentService service;
    private final PlatformProvider platformProvider;
    private final LocaleText localeText;
    private final ImageService imageService;

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
        service.addListener(new LoadTorrentListener() {
            @Override
            public void onStateChanged(State newState) {
                LoaderTorrentComponent.this.onStateChanged(newState);
            }

            @Override
            public void onMediaChanged(Media media) {
                LoaderTorrentComponent.this.onMediaChanged(media);
            }

            @Override
            public void onDownloadStatusChanged(DownloadStatus status) {
                LoaderTorrentComponent.this.onDownloadStatusChanged(status);
            }
        });
    }

    //endregion

    //region Functions

    private void onStateChanged(LoadTorrentListener.State newState) {
        removeRetryButton();

        switch (newState) {
            case INITIALIZING -> onLoadTorrentInitializing();
            case STARTING -> onLoadTorrentStarting();
            case RETRIEVING_SUBTITLES -> onLoadTorrentRetrievingSubtitles();
            case DOWNLOADING_SUBTITLE -> onLoadTorrentDownloadingSubtitle();
            case CONNECTING -> onLoadTorrentConnecting();
            case DOWNLOADING -> onLoadTorrentDownloading();
            case READY -> onLoadTorrentReady();
            case ERROR -> onLoadTorrentError();
        }
    }

    private void onMediaChanged(Media media) {
        loadBackgroundImage(media);
    }

    private void onDownloadStatusChanged(DownloadStatus status) {
        platformProvider.runOnRenderer(() -> {
            progressStatus.setVisible(true);
            progressBar.setProgress(status.getProgress());
            progressBar.setVisible(true);
            statusText.setText(localeText.get(TorrentMessage.DOWNLOADING));
            progressPercentage.setText(ProgressUtils.progressToPercentage(status));
            downloadText.setText(ProgressUtils.progressToDownload(status));
            uploadText.setText(ProgressUtils.progressToUpload(status));
            activePeersText.setText(String.valueOf(status.getSeeds()));
        });
    }

    private void onLoadTorrentStarting() {
        // reset the progress bar to "infinite" animation
        reset();

        platformProvider.runOnRenderer(() -> {
            progressStatus.setVisible(false);
            statusText.setText(localeText.get(TorrentMessage.STARTING));
        });
    }

    private void onLoadTorrentInitializing() {
        platformProvider.runOnRenderer(() -> statusText.setText(localeText.get(TorrentMessage.INITIALIZING)));
    }

    private void onLoadTorrentRetrievingSubtitles() {
        platformProvider.runOnRenderer(() -> statusText.setText(localeText.get(TorrentMessage.RETRIEVING_SUBTITLES)));
    }

    private void onLoadTorrentDownloadingSubtitle() {
        platformProvider.runOnRenderer(() -> statusText.setText(localeText.get(TorrentMessage.DOWNLOADING_SUBTITLE)));
    }

    private void onLoadTorrentConnecting() {
        platformProvider.runOnRenderer(() -> statusText.setText(localeText.get(TorrentMessage.CONNECTING)));
    }

    private void onLoadTorrentDownloading() {
        platformProvider.runOnRenderer(() -> statusText.setText(localeText.get(TorrentMessage.DOWNLOADING)));
    }

    private void onLoadTorrentReady() {
        platformProvider.runOnRenderer(() -> {
            statusText.setText(localeText.get(TorrentMessage.READY));
            progressBar.setProgress(1);
            progressBar.setVisible(true);
        });
    }

    private void onLoadTorrentError() {
        platformProvider.runOnRenderer(() -> {
            // update the actions with the retry button
            loaderActions.getChildren().add(0, loadRetryButton);

            statusText.setText(localeText.get(TorrentMessage.FAILED));
            progressBar.setProgress(1);
            progressBar.setVisible(true);
            progressBar.getStyleClass().add(PROGRESS_ERROR_STYLE_CLASS);
        });
    }

    private void reset() {
        platformProvider.runOnRenderer(() -> {
            statusText.setText(null);
            progressBar.getStyleClass().removeIf(e -> e.equals(PROGRESS_ERROR_STYLE_CLASS));
        });

        resetProgressToDefaultState();
        removeRetryButton();
    }

    private void resetProgressToDefaultState() {
        platformProvider.runOnRenderer(() -> {
            progressBar.setProgress(0.0);
            progressBar.setVisible(false);
            progressBar.getStyleClass().removeIf(e -> e.equals(PROGRESS_ERROR_STYLE_CLASS));
        });
    }

    private void removeRetryButton() {
        platformProvider.runOnRenderer(() -> loaderActions.getChildren().removeIf(e -> e == loadRetryButton));
    }

    private void loadBackgroundImage(Media media) {
        platformProvider.runOnRenderer(() -> backgroundImage.reset());
        imageService.loadFanart(media).whenComplete((bytes, throwable) -> {
            if (throwable == null) {
                platformProvider.runOnRenderer(() ->
                        bytes.ifPresent(e -> backgroundImage.setBackgroundImage(e)));
            } else {
                log.error(throwable.getMessage(), throwable);
            }
        });
    }

    private void close() {
        log.debug("Cancelling torrent loader");
        service.cancel();
        reset();
    }

    @FXML
    private void onCancelClicked(MouseEvent event) {
        event.consume();
        close();
    }

    @FXML
    void onRetryClicked(MouseEvent event) {
        event.consume();
        service.retryLoadingTorrent();
    }

    //endregion
}
