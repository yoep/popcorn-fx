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
import javafx.beans.InvalidationListener;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import javafx.scene.control.Label;
import javafx.scene.control.ListView;
import javafx.scene.image.Image;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;

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
    private Node overlayOrigin;

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
    private Pane qualityButton;
    @FXML
    private Label qualityButtonLabel;
    @FXML
    private Pane overlay;
    private ListView<String> qualityList;

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
        initializeQualityList();
    }

    private void initializePoster() {
        poster.fitHeightProperty().bind(posterHolder.heightProperty());
        poster.fitWidthProperty().bind(posterHolder.widthProperty());
    }

    private void initializePlayButton() {
        playButton.requestFocus();
    }

    private void initializeQualityList() {
        qualityList = new ListView<>();

        qualityList.setMaxWidth(100);
        qualityList.getItems().addListener((InvalidationListener) observable -> qualityList.setMaxHeight(50 * qualityList.getItems().size()));
        qualityList.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> onQualityChanged(newValue));
        qualityList.setOnKeyPressed(this::onOverlayKeyPressed);
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

    @Override
    protected void reset() {
        super.reset();
        Platform.runLater(() -> {
            qualityList.getItems().clear();
            overlay.setVisible(false);
        });
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
        final var qualities = media.getTorrents().get(DEFAULT_TORRENT_AUDIO).keySet().stream()
                .filter(e -> !e.equals("0")) // filter out the 0 quality
                .sorted(Comparator.comparing(o -> Integer.parseInt(o.replaceAll("[a-z]", ""))))
                .collect(Collectors.toList());
        final var defaultQuality = qualities.get(qualities.size() - 1);

        Platform.runLater(() -> {
            qualityList.getItems().clear();
            qualityList.getItems().addAll(qualities);
            qualityList.getSelectionModel().select(defaultQuality);
        });
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

    private void onQuality() {
        switchOverlayItem(qualityButton, qualityList);
        Platform.runLater(() -> qualityList.requestFocus());
    }

    private void onQualityChanged(String newValue) {
        if (StringUtils.isEmpty(newValue))
            return;

        quality = newValue;
        qualityButtonLabel.setText(newValue);
        switchHealth(media.getTorrents().get(DEFAULT_TORRENT_AUDIO).get(newValue));
    }

    private void onOverlayKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            closeOverlay();
        }
    }

    private void onClose() {
        activityManager.register(new CloseDetailsActivity() {
        });
    }

    private void switchOverlayItem(Node origin, Node node) {
        overlayOrigin = origin;
        overlay.getChildren().clear();
        overlay.getChildren().add(node);
        overlay.setVisible(true);
    }

    private void closeOverlay() {
        overlay.setVisible(false);

        if (overlayOrigin != null)
            overlayOrigin.requestFocus();
    }

    @FXML
    private void onDetailsKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.BACK_SPACE || event.getCode().ordinal() == 187) {
            event.consume();
            onClose();
        }
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
    private void onQualityClicked(MouseEvent event) {
        event.consume();
        onQuality();
    }

    @FXML
    private void onQualityKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            onQuality();
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
