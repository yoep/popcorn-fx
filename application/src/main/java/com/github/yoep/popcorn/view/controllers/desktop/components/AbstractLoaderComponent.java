package com.github.yoep.popcorn.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.messages.TorrentMessage;
import com.github.yoep.popcorn.torrent.TorrentService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

@Slf4j
public abstract class AbstractLoaderComponent {
    private static final String PROGRESS_ERROR_STYLE_CLASS = "error";

    protected final LocaleText localeText;
    protected final TorrentService torrentService;

    @FXML
    protected Label statusText;
    @FXML
    private ProgressBar progressBar;

    //region Constructors

    protected AbstractLoaderComponent(LocaleText localeText, TorrentService torrentService) {
        Assert.notNull(localeText, "localeText cannot be null");
        Assert.notNull(torrentService, "torrentService cannot be null");
        this.localeText = localeText;
        this.torrentService = torrentService;
    }

    //endregion

    protected void waitForTorrentStream() {
        Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.INITIALIZING)));

        try {
            if (!torrentService.isInitialized())
                log.trace("Waiting for the torrent service to be initialized");

            while (!torrentService.isInitialized()) {
                Thread.sleep(100);
            }
        } catch (InterruptedException e) {
            log.error("Unexpectedly quit of wait for torrent stream monitor", e);
        }
    }

    protected void updateProgressToErrorState() {
        Platform.runLater(() -> {
            statusText.setText(localeText.get(TorrentMessage.FAILED));
            progressBar.setProgress(1);
            progressBar.setVisible(true);
            progressBar.getStyleClass().add(PROGRESS_ERROR_STYLE_CLASS);
        });
    }

    protected void resetProgress() {
        Platform.runLater(() -> {
            progressBar.setProgress(0.0);
            progressBar.setVisible(false);
            progressBar.getStyleClass().remove(PROGRESS_ERROR_STYLE_CLASS);
        });
    }
}
