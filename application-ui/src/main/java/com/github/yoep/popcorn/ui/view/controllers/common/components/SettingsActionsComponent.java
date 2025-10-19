package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ResetProviderApiRequest;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.SuccessNotificationEvent;
import javafx.fxml.FXML;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import lombok.extern.slf4j.Slf4j;

@Slf4j
public record SettingsActionsComponent(FxChannel fxChannel, ISubtitleService subtitleService, TorrentService torrentService, EventPublisher eventPublisher,
                                       LocaleText localeText) {
    static final String SUBTITLES_CLEANED_MESSAGE = "subtitles_cleaned";
    static final String TORRENTS_CLEANED_MESSAGE = "torrents_cleaned";
    static final String RESET_API_MESSAGE = "reset_api_completed";

    private void onCleanSubtitles() {
        subtitleService.cleanup();
        eventPublisher.publish(new SuccessNotificationEvent(this, localeText.get(SUBTITLES_CLEANED_MESSAGE)));
    }

    private void onCleanTorrents() {
        torrentService.cleanup();
        eventPublisher.publish(new SuccessNotificationEvent(this, localeText.get(TORRENTS_CLEANED_MESSAGE)));
    }

    private void onResetApi() {
        fxChannel.send(ResetProviderApiRequest.newBuilder().setCategory(Media.Category.MOVIES).build());
        fxChannel.send(ResetProviderApiRequest.newBuilder().setCategory(Media.Category.SERIES).build());
        eventPublisher.publish(new SuccessNotificationEvent(this, localeText.get(RESET_API_MESSAGE)));
    }

    @FXML
    void onCleanSubtitlesClicked(MouseEvent event) {
        if (event.getButton() == MouseButton.PRIMARY) {
            event.consume();
            onCleanSubtitles();
        }
    }

    @FXML
    void onCleanSubtitlesPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onCleanSubtitles();
        }
    }

    @FXML
    void onCleanTorrentsClicked(MouseEvent event) {
        if (event.getButton() == MouseButton.PRIMARY) {
            event.consume();
            onCleanTorrents();
        }
    }

    @FXML
    void onCleanTorrentsPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onCleanTorrents();
        }
    }

    @FXML
    void onResetApiClicked(MouseEvent event) {
        if (event.getButton() == MouseButton.PRIMARY) {
            event.consume();
            onResetApi();
        }
    }

    @FXML
    void onResetApiPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onResetApi();
        }
    }
}
