package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamService;
import com.github.yoep.popcorn.backend.adapters.torrent.state.SessionState;
import com.github.yoep.popcorn.ui.messages.TorrentMessage;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import lombok.AccessLevel;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

@Slf4j
@RequiredArgsConstructor(access = AccessLevel.PROTECTED)
public abstract class AbstractLoaderComponent {
    private static final String PROGRESS_ERROR_STYLE_CLASS = "error";

    protected final LocaleText localeText;
    protected final TorrentService torrentService;
    protected final TorrentStreamService torrentStreamService;

    @FXML
    protected Label statusText;
    @FXML
    private ProgressBar progressBar;

    protected synchronized void waitForTorrentStream() {
        Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.INITIALIZING)));

        try {
            if (torrentService.getSessionState() != SessionState.RUNNING)
                log.trace("Waiting for the torrent service to be initialized");

            // add a listener on the session state
            torrentService.sessionStateProperty().addListener((observable, oldValue, newValue) -> onSessionStateChanged(newValue));
            this.wait();
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

    /**
     * Reset the UI elements to there default neutral state.
     */
    protected void reset() {
        runOnFx(() -> statusText.setText(null));
        resetProgress();
    }

    /**
     * Reset the progress bar UI element to it's default state.
     */
    protected void resetProgress() {
        runOnFx(() -> {
            progressBar.setProgress(0.0);
            progressBar.setVisible(false);
            progressBar.getStyleClass().remove(PROGRESS_ERROR_STYLE_CLASS);
        });
    }

    /**
     * Execute the given {@link Runnable} on the JavaFX thread.
     * This will run the executable directly if the current thread is the JavaFX thread, otherwise,
     * it queues the executable for execution on the JavaFX thread.
     *
     * @param runnable The executable to execute.
     */
    protected void runOnFx(Runnable runnable) {
        if (Platform.isFxApplicationThread()) {
            runnable.run();
        } else {
            Platform.runLater(runnable);
        }
    }

    private void onSessionStateChanged(SessionState newValue) {
        log.debug("Session state changed to {}", newValue);
        synchronized (this) {
            this.notifyAll();
        }
    }
}
