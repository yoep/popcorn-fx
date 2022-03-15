package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.ui.messages.TorrentMessage;
import com.github.yoep.popcorn.ui.view.listeners.LoadTorrentListener;
import com.github.yoep.popcorn.ui.view.services.LoadTorrentService;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;

@Slf4j
@ViewController
@RequiredArgsConstructor
public class LoaderUrlComponent {
    static final String PROGRESS_ERROR_STYLE_CLASS = "error";

    private final LoadTorrentService service;
    private final LocaleText localeText;
    private final PlatformProvider platformProvider;

    @FXML
    Label statusText;
    @FXML
    ProgressBar progressBar;

    //region Init

    @PostConstruct
    void init() {
        service.addListener(new LoadTorrentListener() {
            @Override
            public void onStateChanged(State newState) {
                LoaderUrlComponent.this.onStateChanged(newState);
            }

            @Override
            public void onMediaChanged(Media media) {
                // no-op
            }

            @Override
            public void onDownloadStatusChanged(DownloadStatus status) {
                // no-op
            }
        });
    }

    //endregion

    //region Functions

    private void onStateChanged(LoadTorrentListener.State newState) {
        switch (newState) {
            case INITIALIZING -> onLoadTorrentInitializing();
            case STARTING -> onLoadTorrentStarting();
            case CONNECTING -> onLoadTorrentConnecting();
            case ERROR -> onLoadTorrentError();
        }
    }

    private void onLoadTorrentInitializing() {
        platformProvider.runOnRenderer(() -> statusText.setText(localeText.get(TorrentMessage.INITIALIZING)));
    }

    private void onLoadTorrentStarting() {
        reset();
        platformProvider.runOnRenderer(() -> statusText.setText(localeText.get(TorrentMessage.STARTING)));
    }

    private void onLoadTorrentConnecting() {
        platformProvider.runOnRenderer(() -> statusText.setText(localeText.get(TorrentMessage.CONNECTING)));
    }

    private void onLoadTorrentError() {
        platformProvider.runOnRenderer(() -> {
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
    }

    private void close() {
        reset();
        service.cancel();
    }

    @FXML
    private void onCancelClicked() {
        close();
    }

    //endregion
}
