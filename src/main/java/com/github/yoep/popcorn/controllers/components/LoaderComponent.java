package com.github.yoep.popcorn.controllers.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.LoadMovieActivity;
import com.github.yoep.popcorn.activities.PlayVideoActivity;
import com.github.yoep.popcorn.activities.PlayerCloseActivity;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Torrent;
import com.github.yoep.popcorn.messages.TorrentMessage;
import com.github.yoep.popcorn.services.TorrentService;
import com.github.yoep.popcorn.torrent.StreamStatus;
import com.github.yoep.popcorn.torrent.listeners.TorrentListener;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FileUtils;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;

import javax.annotation.PostConstruct;
import java.util.Optional;

@Slf4j
@Component
@RequiredArgsConstructor
public class LoaderComponent {
    private final ActivityManager activityManager;
    private final TaskExecutor taskExecutor;
    private final TorrentService torrentService;
    private final LocaleText localeText;

    private Media media;
    private Torrent mediaTorrent;
    private String quality;

    @FXML
    private Label statusText;
    @FXML
    private Label progressPercentage;
    @FXML
    private Label downloadText;
    @FXML
    private Label uploadText;
    @FXML
    private Label activePeersText;
    @FXML
    private ProgressBar progressBar;
    @FXML
    private Pane progressStatus;

    @PostConstruct
    private void init() {
        activityManager.register(LoadMovieActivity.class, this::startTorrent);
        torrentService.addListener(new TorrentListener() {
            @Override
            public void onLoadError(String message) {
                log.warn("Torrent loading failed: {}", message);
                Platform.runLater(() -> {
                    statusText.setText(localeText.get(TorrentMessage.FAILED));
                    progressBar.setProgress(1);
                    progressBar.getStyleClass().add("error");
                });
            }

            @Override
            public void onStreamStarted(com.github.yoep.popcorn.torrent.Torrent torrent) {
                log.debug("Torrent is starting");
                Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.STARTING)));
            }

            @Override
            public void onStreamError(com.github.yoep.popcorn.torrent.Torrent torrent, Exception e) {
                log.warn("Torrent stream error: " + e.getMessage(), e);
            }

            @Override
            public void onStreamReady(com.github.yoep.popcorn.torrent.Torrent torrent) {
                log.debug("Torrent is ready to be displayed");
                Platform.runLater(() -> {
                    statusText.setText(localeText.get(TorrentMessage.READY));
                    progressBar.setProgress(1);
                });
                activityManager.register(new PlayVideoActivity() {
                    @Override
                    public String getUrl() {
                        return torrent.getVideoFile().getAbsolutePath();
                    }

                    @Override
                    public Optional<String> getQuality() {
                        return Optional.ofNullable(quality);
                    }

                    @Override
                    public Media getMedia() {
                        return media;
                    }

                    @Override
                    public Optional<Torrent> getTorrent() {
                        return Optional.of(mediaTorrent);
                    }
                });
            }

            @Override
            public void onStreamProgress(com.github.yoep.popcorn.torrent.Torrent torrent, StreamStatus status) {
                Platform.runLater(() -> {
                    progressStatus.setVisible(true);
                    progressBar.setProgress(status.getProgress());
                    statusText.setText(localeText.get(TorrentMessage.DOWNLOADING));
                    progressPercentage.setText(String.format("%1$,.2f", status.getProgress() * 100) + "%");
                    downloadText.setText(FileUtils.byteCountToDisplaySize(status.getDownloadSpeed()) + "/s");
                    uploadText.setText(FileUtils.byteCountToDisplaySize(status.getUploadSpeed()) + "/s");
                    activePeersText.setText(String.valueOf(status.getSeeds()));
                });
            }

            @Override
            public void onStreamStopped() {
                log.debug("Torrent stream has stopped");
            }
        });
    }

    private void startTorrent(LoadMovieActivity activity) {
        // store the requested media locally for later use
        this.media = activity.getMedia();
        this.quality = activity.getQuality();

        activity.getTorrent().ifPresentOrElse(
                torrent -> {
                    this.mediaTorrent = torrent;
                    taskExecutor.execute(() -> {
                        Platform.runLater(() -> {
                            progressStatus.setVisible(false);
                            progressBar.setProgress(-1);
                            progressBar.getStyleClass().remove("error");
                        });

                        if (!torrentService.isInitialized())
                            waitForTorrentStream();

                        Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.CONNECTING)));

                        // go to a separate thread to unblock the activityManager
                        taskExecutor.execute(() -> torrentService.startStream(mediaTorrent.getUrl()));
                    });
                },
                () -> {
                    log.error("Failed to load torrent, no torrent present");
                    close();
                });
    }

    private void waitForTorrentStream() {
        Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.INITIALIZING)));

        try {
            while (!torrentService.isInitialized()) {
                Thread.sleep(100);
            }
        } catch (InterruptedException e) {
            log.error("Unexpectedly quit of wait for torrent stream monitor", e);
        }
    }

    private void close() {
        torrentService.stopStream();
        activityManager.register(new PlayerCloseActivity() {
        });
    }

    @FXML
    private void onCancelClicked() {
        close();
    }
}
