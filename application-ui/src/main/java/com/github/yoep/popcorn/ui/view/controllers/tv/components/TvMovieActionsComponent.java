package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.media.providers.Media;
import com.github.yoep.popcorn.backend.media.providers.MediaTorrentInfo;
import com.github.yoep.popcorn.backend.media.providers.MovieDetails;
import com.github.yoep.popcorn.backend.playlists.Playlist;
import com.github.yoep.popcorn.backend.playlists.PlaylistItem;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.font.controls.Icon;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.view.listeners.DetailsComponentListener;
import com.github.yoep.popcorn.ui.view.services.DetailsComponentService;
import com.github.yoep.popcorn.ui.view.services.VideoQualityService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Collections;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class TvMovieActionsComponent extends AbstractActionsComponent {
    static final String DEFAULT_TORRENT_AUDIO = "en";

    private final LocaleText localeText;
    private final DetailsComponentService detailsComponentService;

    private MovieDetails media;

    @FXML
    Button watchNowButton;
    @FXML
    Button watchTrailerButton;
    @FXML
    Button favoriteButton;
    @FXML
    Icon favoriteIcon;

    public TvMovieActionsComponent(EventPublisher eventPublisher, SubtitleService subtitleService, VideoQualityService videoQualityService,
                                   LocaleText localeText, DetailsComponentService detailsComponentService, PlaylistManager playlistManager) {
        super(eventPublisher, subtitleService, videoQualityService, playlistManager);
        this.localeText = localeText;
        this.detailsComponentService = detailsComponentService;
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        super.initialize(url, resourceBundle);
        eventPublisher.register(ShowMovieDetailsEvent.class, event -> {
            onShowMovieDetailsEvent(event);
            return event;
        });
        detailsComponentService.addListener(new DetailsComponentListener() {
            @Override
            public void onWatchChanged(String imdbId, boolean newState) {

            }

            @Override
            public void onLikedChanged(String imdbId, boolean newState) {
                if (media != null && media.getImdbId().equals(imdbId)) {
                    Platform.runLater(() -> updateFavoriteState());
                }
            }
        });
        qualityOverlay.shownProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue) {
                Platform.runLater(() -> qualities.setSelectedItem(videoQualityService.getDefaultVideoResolution(qualities.getItems()), true));
            }
        });
    }

    @Override
    protected Media getMedia() {
        return media;
    }

    @Override
    protected Media getSubItem() {
        return null;
    }

    @Override
    protected Map<String, MediaTorrentInfo> getTorrents() {
        return media.getTorrents().get(DEFAULT_TORRENT_AUDIO);
    }

    @Override
    protected CompletableFuture<List<SubtitleInfo>> retrieveSubtitles() {
        return subtitleService.retrieveSubtitles(media);
    }

    private void onShowMovieDetailsEvent(ShowMovieDetailsEvent event) {
        this.media = event.getMedia();
        var trailer = media.getTrailer();

        Platform.runLater(() -> {
            updateQualities();
            updateFavoriteState();

            watchTrailerButton.setVisible(trailer != null && !trailer.isBlank());
            watchNowButton.requestFocus();
        });
    }

    private void updateFavoriteState() {
        var state = detailsComponentService.isLiked(this.media);

        favoriteButton.setText(localeText.get(state ? DetailsMessage.REMOVE : DetailsMessage.ADD));
        favoriteIcon.setText(state ? Icon.HEART_UNICODE : Icon.HEART_O_UNICODE);
    }

    private void playTrailer() {
        try (var item = PlaylistItem.fromMediaTrailer(media)) {
            playlistManager.play(new Playlist.ByValue(Collections.singletonList(item)));
        }
    }

    private void toggleFavoriteState() {
        detailsComponentService.toggleLikedState(media);
    }

    @FXML
    void onTrailerClicked(MouseEvent event) {
        event.consume();
        playTrailer();
    }

    @FXML
    void onTrailerPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            playTrailer();
        }
    }

    @FXML
    void onFavoriteClicked(MouseEvent event) {
        event.consume();
        toggleFavoriteState();
    }

    @FXML
    void onFavoritePressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            toggleFavoriteState();
        }
    }
}
