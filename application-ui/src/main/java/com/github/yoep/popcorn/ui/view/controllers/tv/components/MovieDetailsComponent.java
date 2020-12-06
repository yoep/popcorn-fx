package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.events.CloseDetailsEvent;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.events.PlayVideoEvent;
import com.github.yoep.popcorn.ui.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.ui.media.favorites.FavoriteService;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.media.providers.models.Movie;
import com.github.yoep.popcorn.ui.media.watched.WatchedService;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.ui.view.services.HealthService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import javafx.scene.control.Label;
import javafx.scene.image.Image;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.event.EventListener;

import java.net.URL;
import java.util.List;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class MovieDetailsComponent extends AbstractTvDetailsComponent<Movie> implements Initializable {
    private static final String DEFAULT_TORRENT_AUDIO = "en";
    private static final String WATCHED_STYLE_CLASS = "watched";
    private static final String LIKED_STYLE_CLASS = "liked";

    private final FavoriteService favoriteService;

    @FXML
    private Node playButton;
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
    private Icon watchedButton;
    @FXML
    private Label watchedText;
    @FXML
    private Icon likeButton;
    @FXML
    private Label likeText;

    //region Constructors

    public MovieDetailsComponent(LocaleText localeText, ImageService imageService, HealthService healthService, SettingsService settingsService,
                                 ApplicationEventPublisher eventPublisher, SubtitleService subtitleService, WatchedService watchedService,
                                 FavoriteService favoriteService) {
        super(localeText, imageService, healthService, settingsService, eventPublisher, subtitleService, watchedService);
        this.favoriteService = favoriteService;
    }

    //endregion

    //region Methods

    @EventListener
    public void onShowMovieDetails(ShowMovieDetailsEvent event) {
        Platform.runLater(() -> load(event.getMedia()));
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        super.initialize(location, resources);
        initializePlayButton();
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
        loadSubtitles();
        initializeWatched();
        initializeFavorite();
    }

    @Override
    protected CompletableFuture<Optional<Image>> loadPoster(Media media) {
        return imageService.loadPoster(media);
    }

    @Override
    protected CompletableFuture<List<SubtitleInfo>> retrieveSubtitles() {
        return subtitleService.retrieveSubtitles(media);
    }

    @Override
    protected void reset() {
        super.reset();
        Platform.runLater(() -> overlay.setVisible(false));
    }

    //endregion

    //region Functions

    @Override
    protected void loadHealth(String quality) {
        switchHealth(media.getTorrents().get(DEFAULT_TORRENT_AUDIO).get(quality));
    }

    private void loadText() {
        title.setText(media.getTitle());
        overview.setText(media.getSynopsis());
        year.setText(media.getYear());
        duration.setText(media.getRuntime() + " min");
        genres.setText(String.join(" / ", media.getGenres()));
    }

    private void loadQualities() {
        var qualities = getVideoResolutions(media.getTorrents().get(DEFAULT_TORRENT_AUDIO));
        var defaultQuality = getDefaultVideoResolution(qualities);

        Platform.runLater(() -> {
            qualityList.getItems().clear();
            qualityList.getItems().addAll(qualities);
            qualityList.getSelectionModel().select(defaultQuality);
        });
    }

    private void initializeWatched() {
        var watched = watchedService.isWatched(media);

        media.setWatched(watched);
        media.watchedProperty().addListener((observable, oldValue, newValue) -> switchWatched(newValue));
        switchWatched(watched);
    }

    private void initializeFavorite() {
        boolean liked = favoriteService.isLiked(media);

        media.setLiked(liked);
        media.likedProperty().addListener((observable, oldValue, newValue) -> switchFavorite(newValue));
        switchFavorite(liked);
    }

    private void onPlay() {
        var mediaTorrentInfo = media.getTorrents().get(DEFAULT_TORRENT_AUDIO).get(quality);

        eventPublisher.publishEvent(new LoadMediaTorrentEvent(this, mediaTorrentInfo, media, quality, subtitle));
    }

    private void onWatchTrailer() {
        eventPublisher.publishEvent(new PlayVideoEvent(this, media.getTrailer(), media.getTitle(), false));
    }

    private void onQuality() {
        overlay.show(qualityButton, qualityList);
    }

    private void onSubtitle() {
        overlay.show(subtitleButton, subtitleList);
    }

    private void onWatchedChanged() {
        boolean watched = !media.isWatched();

        media.setWatched(watched);

        if (watched) {
            watchedService.addToWatchList(media);
        } else {
            watchedService.removeFromWatchList(media);
        }
    }

    private void onLikeChanged() {
        boolean liked = !media.isLiked();

        media.setLiked(liked);

        if (liked) {
            favoriteService.addToFavorites(media);
        } else {
            favoriteService.removeFromFavorites(media);
        }
    }

    private void onClose() {
        eventPublisher.publishEvent(new CloseDetailsEvent(this));
    }

    private void switchWatched(boolean isWatched) {
        Platform.runLater(() -> {
            watchedButton.getStyleClass().removeIf(e -> e.equals(WATCHED_STYLE_CLASS));

            if (isWatched) {
                watchedButton.setText(Icon.EYE_UNICODE);
                watchedButton.getStyleClass().add(WATCHED_STYLE_CLASS);
                watchedText.setText(localeText.get(DetailsMessage.MARK_AS_NOT_SEEN));
            } else {
                watchedButton.setText(Icon.EYE_SLASH_UNICODE);
                watchedText.setText(localeText.get(DetailsMessage.MARK_AS_SEEN));
            }
        });
    }

    private void switchFavorite(boolean isLiked) {
        Platform.runLater(() -> {
            likeButton.getStyleClass().removeIf(e -> e.equals(LIKED_STYLE_CLASS));

            if (isLiked) {
                likeButton.getStyleClass().add(LIKED_STYLE_CLASS);
                likeText.setText(localeText.get(DetailsMessage.REMOVE_FROM_BOOKMARKS));
            } else {
                likeText.setText(localeText.get(DetailsMessage.ADD_TO_BOOKMARKS));
            }
        });
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
            event.consume();
            onQuality();
        }
    }

    @FXML
    private void onSubtitleClicked(MouseEvent event) {
        event.consume();
        onSubtitle();
    }

    @FXML
    private void onSubtitleKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onSubtitle();
        }
    }

    @FXML
    private void onWatchedClicked(MouseEvent event) {
        event.consume();
        onWatchedChanged();
    }

    @FXML
    private void onLikeClicked(MouseEvent event) {
        event.consume();
        onLikeChanged();
    }

    @FXML
    private void onWatchedKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onWatchedChanged();
        }
    }

    @FXML
    private void onLikeKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onLikeChanged();
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
