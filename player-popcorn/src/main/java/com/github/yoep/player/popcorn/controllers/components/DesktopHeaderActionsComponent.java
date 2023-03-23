package com.github.yoep.player.popcorn.controllers.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.player.popcorn.controls.StreamInfo;
import com.github.yoep.player.popcorn.controls.StreamInfoCell;
import com.github.yoep.player.popcorn.listeners.PlayerHeaderListener;
import com.github.yoep.player.popcorn.services.PlayerHeaderService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayerStoppedEvent;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.ResourceBundle;

@Slf4j
@RequiredArgsConstructor
public class DesktopHeaderActionsComponent implements Initializable {
    private final PlayerHeaderService headerService;
    private final EventPublisher eventPublisher;
    private final LocaleText localeText;

    @FXML
    Label quality;
    @FXML
    StreamInfo streamInfo;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeStreamInfo();
        initializeListener();
        eventPublisher.register(PlayerStoppedEvent.class, event -> {
            Platform.runLater(() -> {
                quality.setText(null);
                quality.setVisible(false);
                streamInfo.setVisible(false);
            });
            return event;
        });
    }

    private void initializeListener() {
        headerService.addListener(new PlayerHeaderListener() {
            @Override
            public void onTitleChanged(String title) {
                // no-op
            }

            @Override
            public void onQualityChanged(String quality) {
                DesktopHeaderActionsComponent.this.onQualityChanged(quality);
            }

            @Override
            public void onStreamStateChanged(boolean isStreaming) {
                DesktopHeaderActionsComponent.this.onStreamStateChanged(isStreaming);
            }

            @Override
            public void onDownloadStatusChanged(DownloadStatus progress) {
                DesktopHeaderActionsComponent.this.onDownloadStatusChanged(progress);
            }
        });
    }

    private void initializeStreamInfo() {
        streamInfo.setFactory(cell -> new StreamInfoCell(localeText.get("torrent_" + cell)));
        streamInfo.setVisible(false);
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

    private void closePlayer() {
        eventPublisher.publish(new ClosePlayerEvent(this, ClosePlayerEvent.Reason.USER));
    }

    @FXML
    void onCloseClicked(MouseEvent event) {
        event.consume();
        closePlayer();
    }

    @FXML
    void onClosePressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            closePlayer();
        }
    }
}
