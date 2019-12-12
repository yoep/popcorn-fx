package com.github.yoep.popcorn.controllers.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.PlayMediaMovieActivity;
import com.github.yoep.popcorn.activities.PlayerCloseActivity;
import com.github.yoep.popcorn.media.providers.models.Torrent;
import com.github.yoep.popcorn.messages.TorrentMessage;
import com.github.yoep.popcorn.torrent.StreamStatus;
import com.github.yoep.popcorn.torrent.TorrentStream;
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
import java.util.Map;

@Slf4j
@Component
@RequiredArgsConstructor
public class LoaderComponent {
    private final ActivityManager activityManager;
    private final TaskExecutor taskExecutor;
    private final TorrentStream torrentStream;
    private final LocaleText localeText;

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
        activityManager.register(PlayMediaMovieActivity.class, this::startTorrent);
        torrentStream.addListener(new TorrentListener() {
            @Override
            public void onStreamStarted(com.github.yoep.popcorn.torrent.Torrent torrent) {
                log.debug("Torrent is starting");
                Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.STARTING)));
            }

            @Override
            public void onStreamError(com.github.yoep.popcorn.torrent.Torrent torrent, Exception e) {

            }

            @Override
            public void onStreamReady(com.github.yoep.popcorn.torrent.Torrent torrent) {
                log.debug("Torrent is ready to be displayed");
                Platform.runLater(() -> {
                    statusText.setText(localeText.get(TorrentMessage.READY));
                    progressBar.setProgress(1);
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

            }
        });
    }

    private void startTorrent(PlayMediaMovieActivity activity) {
        taskExecutor.execute(() -> {
            Platform.runLater(() -> {
                progressStatus.setVisible(false);
                progressBar.setProgress(-1);
            });

            if (!torrentStream.isInitialized())
                waitForTorrentStream();

            Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.CONNECTING)));

            // go to a separate thread to unblock the activityManager
            taskExecutor.execute(() -> {
                Map<String, Map<String, Torrent>> torrents = activity.getMedia().getTorrents();

                torrentStream.startStream(torrents.get("en").get(activity.getQuality()).getUrl());
            });
        });
    }

    private void waitForTorrentStream() {
        Platform.runLater(() -> statusText.setText(localeText.get(TorrentMessage.INITIALIZING)));

        try {
            while (!torrentStream.isInitialized()) {
                Thread.sleep(100);
            }
        } catch (InterruptedException e) {
            log.error(e.getMessage(), e);
        }
    }

    private void stopStream() {
        torrentStream.stopStream();
    }

    @FXML
    private void onCancelClicked() {
        stopStream();
        activityManager.register(new PlayerCloseActivity() {
        });
    }
}
