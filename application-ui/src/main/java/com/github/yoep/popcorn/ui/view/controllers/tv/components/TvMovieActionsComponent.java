package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Playlist;
import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.media.MovieDetails;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleInfo;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleService;
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
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

import static com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.TorrentLanguage;
import static com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.TorrentQuality;

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

    public TvMovieActionsComponent(EventPublisher eventPublisher, ISubtitleService subtitleService, VideoQualityService videoQualityService,
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
                // no-op
            }

            @Override
            public void onLikedChanged(String imdbId, boolean newState) {
                if (Objects.equals(media.id(), imdbId)) {
                    Platform.runLater(() -> updateFavoriteState());
                }
            }
        });
        qualityOverlay.shownProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue) {
                videoQualityService.getDefaultVideoResolution(qualities.getItems()).whenComplete((quality, throwable) -> {
                    if (throwable == null) {
                        Platform.runLater(() -> qualities.setSelectedItem(quality, true));
                    } else {
                        log.error("Failed to retrieve video resolution", throwable);
                    }
                });
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
    protected Optional<TorrentQuality> getTorrents() {
        return media.getTorrents().stream()
                .filter(e -> Objects.equals(e.getLanguage(), DEFAULT_TORRENT_AUDIO))
                .map(TorrentLanguage::getTorrents)
                .findFirst();
    }

    @Override
    protected CompletableFuture<List<ISubtitleInfo>> retrieveSubtitles() {
        return subtitleService.retrieveSubtitles(media);
    }

    private void onShowMovieDetailsEvent(ShowMovieDetailsEvent event) {
        this.media = event.getMedia();
        var trailer = media.proto().getTrailer();

        Platform.runLater(() -> {
            updateQualities();
            updateFavoriteState();

            watchTrailerButton.setVisible(trailer != null && !trailer.isBlank());
            watchNowButton.requestFocus();
        });
    }

    private void updateFavoriteState() {
        detailsComponentService.isLiked(this.media).whenComplete((isLiked, throwable) -> {
            if (throwable == null) {
                Platform.runLater(() -> {
                    favoriteButton.setText(localeText.get(isLiked ? DetailsMessage.REMOVE : DetailsMessage.ADD));
                    favoriteIcon.setText(isLiked ? Icon.HEART_UNICODE : Icon.HEART_O_UNICODE);
                });
            } else {
                log.error("Failed to retrieve favorite state", throwable);
            }
        });
    }

    private void playTrailer() {
        playlistManager.play(Playlist.newBuilder()
                .addItems(Playlist.Item.newBuilder()
                        .setUrl(media.proto().getTrailer())
                        .setTitle(media.title())
                        .setCaption("Trailer")
                        .setThumb(media.images().getPoster())
                        .setSubtitlesEnabled(false)
                        .build())
                .build());
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
