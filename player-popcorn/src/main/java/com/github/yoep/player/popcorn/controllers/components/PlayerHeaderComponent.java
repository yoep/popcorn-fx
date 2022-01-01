package com.github.yoep.player.popcorn.controllers.components;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.player.popcorn.controls.StreamInfo;
import com.github.yoep.player.popcorn.controls.StreamInfoCell;
import com.github.yoep.player.popcorn.listeners.PlayerHeaderListener;
import com.github.yoep.player.popcorn.services.PlayerHeaderService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.events.PlayerStoppedEvent;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.input.MouseEvent;
import lombok.RequiredArgsConstructor;
import org.springframework.context.event.EventListener;

import java.net.URL;
import java.util.ResourceBundle;

@ViewController
@RequiredArgsConstructor
public class PlayerHeaderComponent implements Initializable {
    private final PlayerHeaderService headerService;
    private final LocaleText localeText;

    @FXML
    Label title;
    @FXML
    Label quality;
    @FXML
    StreamInfo streamInfo;

    //region Methods

    @EventListener(PlayerStoppedEvent.class)
    public void reset() {
        Platform.runLater(() -> {
            title.setText(null);
            quality.setText(null);
            quality.setVisible(false);
            streamInfo.setVisible(false);
        });
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeStreamInfo();
        initializeListener();
    }

    private void initializeStreamInfo() {
        streamInfo.setFactory(cell -> new StreamInfoCell(localeText.get("torrent_" + cell)));
        streamInfo.setVisible(false);
    }

    private void initializeListener() {
        headerService.addListener(new PlayerHeaderListener() {
            @Override
            public void onTitleChanged(String title) {
                PlayerHeaderComponent.this.onTitleChanged(title);
            }

            @Override
            public void onQualityChanged(String quality) {
                PlayerHeaderComponent.this.onQualityChanged(quality);
            }

            @Override
            public void onStreamStateChanged(boolean isStreaming) {
                PlayerHeaderComponent.this.onStreamStateChanged(isStreaming);
            }

            @Override
            public void onDownloadStatusChanged(DownloadStatus progress) {
                PlayerHeaderComponent.this.onDownloadStatusChanged(progress);
            }
        });
    }

    //endregion

    //region Functions

    private void onTitleChanged(String title) {
        Platform.runLater(() -> this.title.setText(title));
    }

    private void onQualityChanged(String quality) {
        Platform.runLater(() -> {
            this.quality.setText(quality);
            this.quality.setVisible(true);
        });
    }

    private void onStreamStateChanged(boolean isStreaming) {
        Platform.runLater(() -> this.streamInfo.setVisible(isStreaming));
    }

    private void onDownloadStatusChanged(DownloadStatus progress) {
        Platform.runLater(() -> this.streamInfo.update(progress));
    }

    @FXML
    void close(MouseEvent event) {
        event.consume();
        headerService.stop();
    }

    //endregion
}
