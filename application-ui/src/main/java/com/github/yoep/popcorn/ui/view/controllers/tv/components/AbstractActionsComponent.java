package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowDetailsEvent;
import com.github.yoep.popcorn.backend.media.Episode;
import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.media.MovieDetails;
import com.github.yoep.popcorn.backend.media.ShowDetails;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.subtitles.SubtitleHelper;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleService;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import com.github.yoep.popcorn.ui.view.services.VideoQualityService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Button;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.List;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

import static com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.TorrentQuality;

@Slf4j
@RequiredArgsConstructor
public abstract class AbstractActionsComponent implements Initializable {
    protected final EventPublisher eventPublisher;
    protected final ISubtitleService subtitleService;
    protected final VideoQualityService videoQualityService;
    protected final PlaylistManager playlistManager;

    private CompletableFuture<List<ISubtitleInfo>> subtitleFuture;

    @FXML
    Overlay qualityOverlay;
    @FXML
    AxisItemSelection<String> qualities;
    @FXML
    Overlay subtitleOverlay;
    @FXML
    AxisItemSelection<ISubtitleInfo> subtitles;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        eventPublisher.register(ShowDetailsEvent.class, event -> {
            Platform.runLater(() -> {
                qualityOverlay.hide();
                subtitleOverlay.hide();
            });
            return event;
        });

        subtitleService.getDefaultOrInterfaceLanguage(subtitles.getItems()).thenAccept(subtitle -> {
            qualities.setOnItemActivated(newValue -> {
                qualityOverlay.hide();
                subtitleOverlay.show();
                subtitles.setSelectedItem(subtitle, true);
            });
        });

        subtitles.setOnItemActivated(subtitle -> {
            subtitleOverlay.hide();
            subtitleService.updatePreferredLanguage(subtitle.getLanguage());
            play();
        });
        subtitles.setItemFactory(item -> new Button(SubtitleHelper.getNativeName(item.getLanguage())));
    }

    /**
     * Retrieve the currently displayed media item.
     */
    protected abstract Media getMedia();

    /**
     * Retrieve the sub item of the media.
     */
    protected abstract Media getSubItem();

    /**
     * Retrieve the available torrents for the current media.
     */
    protected abstract Optional<TorrentQuality> getTorrents();

    /**
     * Retrieve the subtitle for the current media item.
     */
    protected abstract CompletableFuture<List<ISubtitleInfo>> retrieveSubtitles();

    protected void updateQualities() {
        getTorrents().ifPresent(torrents -> qualities.setItems(videoQualityService.getVideoResolutions(torrents)));
        updateSubtitles();
    }

    private void play() {
        if (getMedia() instanceof MovieDetails movie) {
            playlistManager.play(movie, qualities.getSelectedItem());
        } else if (getMedia() instanceof ShowDetails show) {
            var episode = (Episode) getSubItem();
            playlistManager.play(show, episode, qualities.getSelectedItem());
        } else {
            log.warn("Unable to start playback, unsupported media type {}", getMedia().getClass().getSimpleName());
        }
    }

    private void updateSubtitles() {
        if (subtitleFuture != null)
            subtitleFuture.cancel(true);

        subtitleService.defaultSubtitles().thenAccept(defaultSubtitles -> {
            Platform.runLater(() -> subtitles.setItems(defaultSubtitles.toArray(new ISubtitleInfo[0])));

            retrieveSubtitles().thenAccept(subtitles ->
                    Platform.runLater(() -> this.subtitles.setItems(subtitles.toArray(new ISubtitleInfo[0]))));
        });
    }
}
