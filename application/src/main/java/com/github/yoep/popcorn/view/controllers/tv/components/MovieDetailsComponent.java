package com.github.yoep.popcorn.view.controllers.tv.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.media.providers.models.Movie;
import com.github.yoep.popcorn.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.popcorn.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.image.Image;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.Comparator;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.stream.Collectors;

@Slf4j
public class MovieDetailsComponent extends AbstractTvDetailsComponent<Movie> implements Initializable {
    private static final String DEFAULT_TORRENT_AUDIO = "en";

    private final ActivityManager activityManager;

    private String quality;
    private SubtitleInfo subtitle;

    @FXML
    private Icon playButton;
    @FXML
    private Label title;
    @FXML
    private Label overview;
    @FXML
    private Label year;
    @FXML
    private Label duration;
    @FXML
    private Label genres;
    @FXML
    private Label qualityButton;

    //region Constructors

    public MovieDetailsComponent(ActivityManager activityManager, TorrentService torrentService, ImageService imageService) {
        super(imageService, torrentService);
        this.activityManager = activityManager;
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializePoster();
        initializePlayButton();
    }

    private void initializePoster() {
        poster.fitHeightProperty().bind(posterHolder.heightProperty());
        poster.fitWidthProperty().bind(posterHolder.widthProperty());
    }

    private void initializePlayButton() {
        playButton.requestFocus();
    }

    //endregion

    //region AbstractTvDetailsComponent

    @Override
    protected void load(Movie media) {
        super.load(media);

        loadText();
        loadQualities();
    }

    @Override
    protected CompletableFuture<Optional<Image>> loadPoster(Media media) {
        return imageService.loadPoster(media);
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        activityManager.register(ShowMovieDetailsActivity.class, activity ->
                Platform.runLater(() -> load(activity.getMedia())));
    }

    //endregion

    //region Functions

    private void loadText() {
        title.setText(media.getTitle());
        overview.setText(media.getSynopsis());
        year.setText(media.getYear());
        duration.setText(media.getRuntime() + " min");
        genres.setText(String.join(" / ", media.getGenres()));
    }

    private void loadQualities() {
        var qualities = media.getTorrents().get(DEFAULT_TORRENT_AUDIO).keySet().stream()
                .filter(e -> !e.equals("0")) // filter out the 0 quality
                .sorted(Comparator.comparing(o -> Integer.parseInt(o.replaceAll("[a-z]", ""))))
                .collect(Collectors.toList());
        var defaultQuality = qualities.get(qualities.size() - 1);

        qualityButton.setText(defaultQuality);
        quality = defaultQuality;

        switchHealth(media.getTorrents().get(DEFAULT_TORRENT_AUDIO).get(defaultQuality));
    }

    private void onPlay() {
        activityManager.register(new LoadMediaTorrentActivity() {
            @Override
            public String getQuality() {
                return quality;
            }

            @Override
            public Media getMedia() {
                return media;
            }

            @Override
            public MediaTorrentInfo getTorrent() {
                return media.getTorrents().get(DEFAULT_TORRENT_AUDIO).get(quality);
            }

            @Override
            public Optional<SubtitleInfo> getSubtitle() {
                return Optional.ofNullable(subtitle);
            }
        });
    }

    private void onWatchTrailer() {
        activityManager.register(new PlayVideoActivity() {
            @Override
            public String getUrl() {
                return media.getTrailer();
            }

            @Override
            public String getTitle() {
                return media.getTitle();
            }

            @Override
            public boolean isSubtitlesEnabled() {
                return false;
            }
        });
    }

    private void onClose() {
        activityManager.register(new CloseDetailsActivity() {
        });
    }

    @FXML
    private void onPlayClicked(MouseEvent event) {
        event.consume();
        onPlay();
    }

    @FXML
    private void onPlayKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onPlay();
        }
    }

    @FXML
    private void onWatchTrailerClicked(MouseEvent event) {
        event.consume();
        onWatchTrailer();
    }

    @FXML
    private void onWatchTrailerKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onWatchTrailer();
        }
    }

    @FXML
    private void onCloseClicked(MouseEvent event) {
        event.consume();
        onClose();
    }

    @FXML
    private void onCloseKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onClose();
        }
    }

    //endregion
}
