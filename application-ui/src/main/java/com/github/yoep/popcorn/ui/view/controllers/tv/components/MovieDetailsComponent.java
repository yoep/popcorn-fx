package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.events.CloseDetailsEvent;
import com.github.yoep.popcorn.ui.events.LoadMediaTorrentEvent;
import com.github.yoep.popcorn.ui.events.PlayVideoEvent;
import com.github.yoep.popcorn.ui.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.ui.media.favorites.FavoriteService;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.media.providers.models.MediaTorrentInfo;
import com.github.yoep.popcorn.ui.media.providers.models.Movie;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleInfo;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.torrent.adapter.TorrentService;
import javafx.application.Platform;
import javafx.beans.InvalidationListener;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
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
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class MovieDetailsComponent extends AbstractTvDetailsComponent<Movie> implements Initializable {
    private static final String DEFAULT_TORRENT_AUDIO = "en";
    private static final String LIKED_STYLE_CLASS = "liked";
    private static final String SUBTITLE_LOADING_STYLE_CLASS = "loading";
    private static final String SUBTITLE_SUCCESS_STYLE_CLASS = "success";
    private static final String SUBTITLE_FAILED_STYLE_CLASS = "failed";

    private final ActivityManager activityManager;
    private final SubtitleService subtitleService;
    private final FavoriteService favoriteService;

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
    private Icon subtitleStatus;
    @FXML
    private Pane qualityButton;
    @FXML
    private Label qualityButtonLabel;
    @FXML
    private Icon likeButton;
    @FXML
    private Label likeText;
    @FXML
    private Overlay overlay;
    private ListView<String> qualityList;

    //region Constructors

    public MovieDetailsComponent(LocaleText localeText,
                                 ActivityManager activityManager,
                                 SubtitleService subtitleService,
                                 FavoriteService favoriteService,
                                 TorrentService torrentService,
                                 ImageService imageService,
                                 SettingsService settingsService) {
        super(localeText, imageService, torrentService, settingsService);
        this.activityManager = activityManager;
        this.subtitleService = subtitleService;
        this.favoriteService = favoriteService;
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializePlayButton();
        initializeQualityList();
    }

    private void initializePlayButton() {
        playButton.requestFocus();
    }

    private void initializeQualityList() {
        qualityList = new ListView<>();

        qualityList.setMaxWidth(100);
        qualityList.getItems().addListener((InvalidationListener) observable -> qualityList.setMaxHeight(50.0 * qualityList.getItems().size()));
        qualityList.getSelectionModel().selectedItemProperty().addListener((observable, oldValue, newValue) -> onQualityChanged(newValue));
    }

    //endregion

    //region AbstractTvDetailsComponent

    @Override
    protected void load(Movie media) {
        super.load(media);

        loadText();
        loadQualities();
        loadSubtitles();
        initializeFavorite();
    }

    @Override
    protected CompletableFuture<Optional<Image>> loadPoster(Media media) {
        return imageService.loadPoster(media);
    }

    @Override
    protected void reset() {
        super.reset();
        Platform.runLater(() -> {
            subtitleStatus.getStyleClass().remove(SUBTITLE_LOADING_STYLE_CLASS);
            qualityList.getItems().clear();
            overlay.setVisible(false);
        });
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        activityManager.register(ShowMovieDetailsEvent.class, activity ->
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
        var qualities = getVideoResolutions(media.getTorrents().get(DEFAULT_TORRENT_AUDIO));
        var defaultQuality = getDefaultVideoResolution(qualities);

        Platform.runLater(() -> {
            qualityList.getItems().clear();
            qualityList.getItems().addAll(qualities);
            qualityList.getSelectionModel().select(defaultQuality);
        });
    }

    private void loadSubtitles() {
        subtitleStatus.getStyleClass().add(SUBTITLE_LOADING_STYLE_CLASS);
        subtitleService.retrieveSubtitles(media).whenComplete((subtitleInfos, throwable) -> {
            subtitleStatus.getStyleClass().remove(SUBTITLE_LOADING_STYLE_CLASS);

            if (throwable == null) {
                subtitle = subtitleService.getDefaultOrInterfaceLanguage(subtitleInfos);

                if (subtitle.isNone()) {
                    subtitleStatus.getStyleClass().add(SUBTITLE_FAILED_STYLE_CLASS);
                } else {
                    subtitleStatus.getStyleClass().add(SUBTITLE_SUCCESS_STYLE_CLASS);
                }
            } else {
                log.error(throwable.getMessage(), throwable);
                subtitleStatus.getStyleClass().add(SUBTITLE_FAILED_STYLE_CLASS);
            }
        });
    }

    private void initializeFavorite() {
        boolean liked = favoriteService.isLiked(media);

        media.setLiked(liked);
        media.likedProperty().addListener((observable, oldValue, newValue) -> switchFavorite(newValue));
        switchFavorite(liked);
    }

    private void onPlay() {
        activityManager.register(new LoadMediaTorrentEvent() {
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
        activityManager.register(new PlayVideoEvent() {
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
        overlay.show(qualityButton, qualityList);
    }

    private void onQualityChanged(String newValue) {
        if (StringUtils.isEmpty(newValue))
            return;

        quality = newValue;
        qualityButtonLabel.setText(newValue);
        switchHealth(media.getTorrents().get(DEFAULT_TORRENT_AUDIO).get(newValue));
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
        activityManager.register(new CloseDetailsEvent() {
        });
    }

    private void switchFavorite(boolean isLiked) {
        Platform.runLater(() -> {
            likeButton.getStyleClass().removeIf(e -> e.equals(LIKED_STYLE_CLASS));

            if (isLiked) {
                likeButton.getStyleClass().add(LIKED_STYLE_CLASS);
                likeText.setText(localeText.get(DetailsMessage.UNFAVORED));
            } else {
                likeText.setText(localeText.get(DetailsMessage.FAVORITE));
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
    private void onLikeClicked(MouseEvent event) {
        event.consume();
        onLikeChanged();
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
