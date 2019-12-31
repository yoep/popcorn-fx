package com.github.yoep.popcorn.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.favorites.FavoriteService;
import com.github.yoep.popcorn.media.providers.models.Episode;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Movie;
import com.github.yoep.popcorn.media.providers.models.TorrentInfo;
import com.github.yoep.popcorn.messages.DetailsMessage;
import com.github.yoep.popcorn.subtitle.SubtitleService;
import com.github.yoep.popcorn.subtitle.controls.LanguageFlagCell;
import com.github.yoep.popcorn.subtitle.models.Language;
import com.github.yoep.popcorn.subtitle.models.SubtitleInfo;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.popcorn.watched.WatchedService;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import java.io.IOException;
import java.net.URL;
import java.util.Optional;
import java.util.ResourceBundle;

@Slf4j
@Component
public class MovieDetailsComponent extends AbstractDetailsComponent<Movie> {
    private static final String DEFAULT_TORRENT_AUDIO = "en";
    private static final String WATCHED_STYLE_CLASS = "seen";

    private final ActivityManager activityManager;
    private final FavoriteService favoriteService;
    private final WatchedService watchedService;
    private final SubtitleService subtitleService;

    private boolean watched;

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
    private Icon watchedIcon;
    @FXML
    private Label watchedText;
    @FXML
    private Button watchTrailerButton;

    //region Constructors

    public MovieDetailsComponent(ActivityManager activityManager,
                                 LocaleText localeText,
                                 Application application,
                                 TaskExecutor taskExecutor,
                                 TorrentService torrentService,
                                 FavoriteService favoriteService,
                                 WatchedService watchedService, SubtitleService subtitleService) {
        super(taskExecutor, localeText, torrentService, application);
        this.activityManager = activityManager;
        this.favoriteService = favoriteService;
        this.watchedService = watchedService;
        this.subtitleService = subtitleService;
    }

    //endregion

    //region Methods

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializePoster();
        initializeTooltips();
        initializeLanguageSelection();
    }

    //endregion

    //region AbstractDetailsComponent

    @Override
    protected void reset() {
        super.reset();
        resetLanguageSelection();

        title.setText(StringUtils.EMPTY);
        overview.setText(StringUtils.EMPTY);
        year.setText(StringUtils.EMPTY);
        duration.setText(StringUtils.EMPTY);
        genres.setText(StringUtils.EMPTY);
        favoriteIcon.getStyleClass().remove(LIKED_STYLE_CLASS);
        watchedIcon.getStyleClass().remove(WATCHED_STYLE_CLASS);
        qualitySelectionPane.getChildren().clear();
        poster.setImage(null);
    }

    //endregion

    //region Functions

    @PostConstruct
    private void init() {
        initializeListeners();
    }

    private void initializeTooltips() {
        Tooltip tooltip = new Tooltip(localeText.get(DetailsMessage.MAGNET_LINK));
        setInstantTooltip(tooltip);
        Tooltip.install(magnetLink, tooltip);
    }

    private void initializeListeners() {
        activityManager.register(ShowMovieDetailsActivity.class, activity ->
                Platform.runLater(() -> load(activity.getMedia())));
    }

    private void initializeLanguageSelection() {
        languageSelection.setFactory(new LanguageFlagCell() {
            @Override
            public void updateItem(SubtitleInfo item) {
                if (item == null)
                    return;

                setText(null);
                item.getFlagResource().ifPresent(e -> {
                    try {
                        var image = new ImageView(new Image(e.getInputStream()));

                        image.setFitHeight(20);
                        image.setPreserveRatio(true);

                        Tooltip tooltip = new Tooltip(Language.valueOf(item.getLanguage()).getNativeName());
                        setInstantTooltip(tooltip);
                        Tooltip.install(image, tooltip);

                        setGraphic(image);
                    } catch (IOException ex) {
                        log.error(ex.getMessage(), ex);
                    }
                });
            }
        });

        languageSelection.addListener(newValue -> this.subtitle = newValue);
        resetLanguageSelection();
    }

    private void load(Movie media) {
        Assert.notNull(media, "media cannot be null");
        this.media = media;

        loadSubtitles();
        loadText();
        loadStars();
        loadButtons();
        loadFavoriteAndWatched();
        loadQualitySelection(media.getTorrents().get(DEFAULT_TORRENT_AUDIO));
        loadPosterImage();
    }

    private void loadText() {
        title.setText(media.getTitle());
        overview.setText(media.getSynopsis());
        year.setText(media.getYear());
        duration.setText(media.getRuntime() + " min");
        genres.setText(String.join(" / ", media.getGenres()));
    }

    private void loadButtons() {
        watchTrailerButton.setVisible(StringUtils.isNotEmpty(media.getTrailer()));
    }

    private void loadFavoriteAndWatched() {
        switchFavorite(favoriteService.isFavorite(media));
        switchWatched(watchedService.isWatched(media));
    }

    private void loadSubtitles() {
        resetLanguageSelection();
        subtitleService.retrieveSubtitles(media).whenComplete(this::handleSubtitlesResponse);
    }

    @Override
    protected void switchActiveQuality(String quality) {
        super.switchActiveQuality(quality);
        switchHealth(media.getTorrents().get(DEFAULT_TORRENT_AUDIO).get(quality));
    }

    private void switchWatched(boolean isWatched) {
        this.watched = isWatched;

        if (isWatched) {
            watchedIcon.setText(Icon.CHECK_UNICODE);
            watchedIcon.getStyleClass().add(WATCHED_STYLE_CLASS);
            watchedText.setText(localeText.get(DetailsMessage.SEEN));
        } else {
            watchedIcon.setText(Icon.EYE_SLASH_UNICODE);
            watchedIcon.getStyleClass().remove(WATCHED_STYLE_CLASS);
            watchedText.setText(localeText.get(DetailsMessage.NOT_SEEN));
        }
    }

    @FXML
    private void onMagnetClicked(MouseEvent event) {
        TorrentInfo torrentInfo = media.getTorrents().get(DEFAULT_TORRENT_AUDIO).get(quality);

        if (event.getButton() == MouseButton.SECONDARY) {
            copyMagnetLink(torrentInfo);
        } else {
            openMagnetLink(torrentInfo);
        }
    }

    @FXML
    private void onWatchNowClicked() {
        activityManager.register(new LoadTorrentActivity() {
            @Override
            public String getQuality() {
                return quality;
            }

            @Override
            public Media getMedia() {
                return media;
            }

            @Override
            public TorrentInfo getTorrent() {
                return media.getTorrents().get(DEFAULT_TORRENT_AUDIO).get(quality);
            }

            @Override
            public Optional<Episode> getEpisode() {
                return Optional.empty();
            }

            @Override
            public Optional<SubtitleInfo> getSubtitle() {
                return Optional.ofNullable(subtitle);
            }
        });
    }

    @FXML
    private void onTrailerClicked() {
        activityManager.register(new PlayVideoActivity() {
            @Override
            public String getUrl() {
                return media.getTrailer();
            }

            @Override
            public Optional<String> getQuality() {
                return Optional.empty();
            }

            @Override
            public Optional<Episode> getEpisode() {
                return Optional.empty();
            }

            @Override
            public Media getMedia() {
                return media;
            }

            @Override
            public Optional<SubtitleInfo> getSubtitle() {
                return Optional.empty();
            }
        });
    }

    @FXML
    private void onSubtitleLabelClicked() {
        languageSelection.show();
    }

    @FXML
    private void onFavoriteClicked() {
        boolean newValue = !liked;

        switchFavorite(newValue);

        if (newValue) {
            favoriteService.addToFavorites(media);
        } else {
            favoriteService.removeFromFavorites(media);
        }
    }

    @FXML
    private void onWatchedClicked() {
        boolean newValue = !watched;

        switchWatched(newValue);

        if (newValue) {
            watchedService.addToWatchList(media);
        } else {
            watchedService.removeFromWatchList(media);
        }
    }

    @FXML
    private void close() {
        reset();
        activityManager.register(new CloseDetailsActivity() {
        });
    }

    //endregion
}
