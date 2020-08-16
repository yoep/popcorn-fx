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
import org.springframework.context.event.EventListener;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class PlayerHeaderComponent implements Initializable {
    private final VideoPlayerService videoPlayerService;
    private final LocaleText localeText;

    private final TorrentListener torrentListener = createTorrentListener();

    @FXML
    private Label title;
    @FXML
    private Label quality;
    @FXML
    private StreamInfo streamInfo;

    //region Methods

    @EventListener
    public void onPlayVideo(PlayVideoEvent event) {
        // set the title of the video as it should be always present
        Platform.runLater(() -> {
            this.title.setText(event.getTitle());
            this.quality.setVisible(false);
        });
    }

    @EventListener
    public void onPlayMedia(PlayMediaEvent event) {
        Platform.runLater(() -> {
            this.quality.setText(event.getQuality());
            this.quality.setVisible(true);
        });
    }

    @EventListener
    public void onPlayTorrent(PlayTorrentEvent event) {
        showTorrentProgress(event.getTorrent());
    }

    //endregion

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

    //region Functions

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
