package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayVideoEvent;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;

import java.net.URL;
import java.util.List;
import java.util.Map;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class TvMovieActionsComponent extends AbstractActionsComponent {
    static final String DEFAULT_TORRENT_AUDIO = "en";

    private MovieDetails media;

    @FXML
    Button watchNowButton;
    @FXML
    Button watchTrailerButton;

    public TvMovieActionsComponent(EventPublisher eventPublisher, SubtitleService subtitleService) {
        super(eventPublisher, subtitleService);
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        super.initialize(url, resourceBundle);
        eventPublisher.register(ShowMovieDetailsEvent.class, event -> {
            onShowMovieDetailsEvent(event);
            return event;
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
        Platform.runLater(() -> {
            updateQualities();
            watchTrailerButton.setVisible(StringUtils.isNotEmpty(media.getTrailer()));
            watchNowButton.requestFocus();
        });
    }

    private void playTrailer() {
        eventPublisher.publish(new PlayVideoEvent(this, media.getTrailer(), media.getTitle(), false, media.getImages().getFanart()));
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
}
