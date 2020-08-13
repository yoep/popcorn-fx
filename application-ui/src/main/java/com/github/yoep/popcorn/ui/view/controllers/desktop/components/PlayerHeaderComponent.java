package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.events.ClosePlayerEvent;
import com.github.yoep.popcorn.ui.events.PlayMediaEvent;
import com.github.yoep.popcorn.ui.events.PlayTorrentEvent;
import com.github.yoep.popcorn.ui.events.PlayVideoEvent;
import com.github.yoep.popcorn.ui.torrent.controls.StreamInfo;
import com.github.yoep.popcorn.ui.torrent.controls.StreamInfoCell;
import com.github.yoep.popcorn.ui.view.services.VideoPlayerService;
import com.github.yoep.torrent.adapter.listeners.AbstractTorrentListener;
import com.github.yoep.torrent.adapter.listeners.TorrentListener;
import com.github.yoep.torrent.adapter.model.DownloadStatus;
import com.github.yoep.torrent.adapter.model.Torrent;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class PlayerHeaderComponent implements Initializable {
    private final ActivityManager activityManager;
    private final VideoPlayerService videoPlayerService;
    private final LocaleText localeText;

    private final TorrentListener torrentListener = createTorrentListener();

    @FXML
    private Label title;
    @FXML
    private Label quality;
    @FXML
    private StreamInfo streamInfo;

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeStreamInfo();
    }

    private void initializeStreamInfo() {
        streamInfo.setFactory(cell -> new StreamInfoCell(localeText.get("torrent_" + cell)));
        streamInfo.setVisible(false);
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializeActivityListeners();
    }

    private void initializeActivityListeners() {
        activityManager.register(PlayVideoEvent.class, this::onPlayVideo);
        activityManager.register(ClosePlayerEvent.class, this::onClose);
    }

    //endregion

    //region Functions

    private void onPlayVideo(PlayVideoEvent activity) {
        // set the title of the video as it should be always present
        Platform.runLater(() -> {
            this.title.setText(activity.getTitle());
            this.quality.setVisible(false);
        });

        // check if the video contains media information
        // if so, update additional information of the media
        if (activity instanceof PlayMediaEvent) {
            var mediaActivity = (PlayMediaEvent) activity;
            onPlayMedia(mediaActivity);
        }

        // check if the activity contains torrent information
        if (PlayTorrentEvent.class.isAssignableFrom(activity.getClass())) {
            var torrentActivity = (PlayTorrentEvent) activity;
            var torrent = torrentActivity.getTorrent();

            showTorrentProgress(torrent);
        }
    }

    private void onPlayMedia(PlayMediaEvent activity) {
        Platform.runLater(() -> {
            this.quality.setText(activity.getQuality());
            this.quality.setVisible(true);
        });
    }

    private void onClose(ClosePlayerEvent activity) {
        reset();
    }

    private void reset() {
        Platform.runLater(() -> {
            title.setText(null);
            quality.setText(null);
            quality.setVisible(false);
            streamInfo.setVisible(false);
        });
    }

    private void showTorrentProgress(Torrent torrent) {
        Platform.runLater(() -> this.streamInfo.setVisible(true));
        torrent.addListener(torrentListener);
    }

    private TorrentListener createTorrentListener() {
        return new AbstractTorrentListener() {
            @Override
            public void onDownloadProgress(DownloadStatus status) {
                streamInfo.update(status);
            }
        };
    }

    @FXML
    private void close() {
        videoPlayerService.close();
    }

    //endregion
}
