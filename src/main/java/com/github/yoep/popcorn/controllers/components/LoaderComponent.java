package com.github.yoep.popcorn.controllers.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.PlayMediaMovieActivity;
import com.github.yoep.popcorn.activities.PlayerCloseActivity;
import com.github.yoep.popcorn.media.providers.models.Torrent;
import com.github.yoep.popcorn.messages.TorrentMessage;
import com.github.yoep.popcorn.torrent.TorrentStream;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
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
    }

    private void startTorrent(PlayMediaMovieActivity activity) {
        Platform.runLater(() -> {
            progressStatus.setVisible(false);
            statusText.setText(localeText.get(TorrentMessage.CONNECTING));
        });

        // go to a separate thread to unblock the activityManager
        taskExecutor.execute(() -> {
            Map<String, Map<String, Torrent>> torrents = activity.getMedia().getTorrents();

            torrentStream.startStream(torrents.get("en").get(activity.getQuality()).getUrl());
        });
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
