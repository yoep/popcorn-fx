package com.github.yoep.popcorn.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.favorites.FavoriteService;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Movie;
import com.github.yoep.popcorn.media.providers.models.Torrent;
import com.github.yoep.popcorn.messages.DetailsMessage;
import com.github.yoep.popcorn.subtitle.controls.LanguageSelection;
import com.github.yoep.popcorn.subtitle.models.Subtitle;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.popcorn.torrent.models.TorrentHealth;
import com.github.yoep.popcorn.watched.WatchedService;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.input.Clipboard;
import javafx.scene.input.ClipboardContent;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.Comparator;
import java.util.List;
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.stream.Collectors;
import java.util.stream.Stream;

@Slf4j
@Component
public class MovieDetailsComponent extends AbstractDetailsComponent<Movie> {
    private static final String DEFAULT_TORRENT_AUDIO = "en";
    private static final String LIKED_STYLE_CLASS = "liked";
    private static final String WATCHED_STYLE_CLASS = "seen";

    private final ActivityManager activityManager;
    private final LocaleText localeText;
    private final Application application;
    private final TorrentService torrentService;
    private final FavoriteService favoriteService;
    private final WatchedService watchedService;

    private String quality;
    private boolean liked;
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
    private Icon magnetLink;
    @FXML
    private Icon health;
    @FXML
    private Icon favoriteIcon;
    @FXML
    private Label favoriteText;
    @FXML
    private Icon watchedIcon;
    @FXML
    private Label watchedText;
    @FXML
    private Button watchTrailerButton;
    @FXML
    private Pane qualitySelectionPane;
    @FXML
    private LanguageSelection<Subtitle> languageSelection;

    //region Constructors

    public MovieDetailsComponent(ActivityManager activityManager,
                                 LocaleText localeText,
                                 Application application,
                                 TaskExecutor taskExecutor,
                                 TorrentService torrentService,
                                 FavoriteService favoriteService,
                                 WatchedService watchedService) {
        super(taskExecutor);
        this.activityManager = activityManager;
        this.localeText = localeText;
        this.application = application;
        this.torrentService = torrentService;
        this.favoriteService = favoriteService;
        this.watchedService = watchedService;
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
        activityManager.register(SubtitlesRetrievedActivity.class, this::loadSubtitles);
    }

    private void initializeLanguageSelection() {
        //        ListCell<Subtitle> cell = new ListCell<>() {
        //            @Override
        //            protected void updateItem(Subtitle item, boolean empty) {
        //                super.updateItem(item, empty);
        //
        //                if (!empty && item.getFlagResource().isPresent()) {
        //                    try {
        //                        Image image = new Image(item.getFlagResource().get().getInputStream());
        //                        ImageView imageView = new ImageView(image);
        //                        imageView.setFitHeight(20);
        //                        imageView.setPreserveRatio(true);
        //
        //                        setText("a");
        //                        setGraphic(imageView);
        //                    } catch (IOException ex) {
        //                        log.error(ex.getMessage(), ex);
        //                    }
        //                }
        //            }
        //        };
        //
        //        languageSelection.setCellFactory(param -> cell);
        //        languageSelection.setButtonCell(cell);
    }

    private void reset() {
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

    private void load(Movie media) {
        Assert.notNull(media, "media cannot be null");
        this.media = media;

        reset();
        loadText();
        loadStars();
        loadButtons();
        loadQualitySelection();
        loadFavoriteAndWatched();
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

    private void loadQualitySelection() {
        List<Label> qualities = media.getTorrents().get(DEFAULT_TORRENT_AUDIO).keySet().stream()
                .sorted(Comparator.comparing(o -> Integer.parseInt(o.replaceAll("[a-z]", ""))))
                .map(this::createQualityOption)
                .collect(Collectors.toList());

        qualitySelectionPane.getChildren().addAll(qualities);
        switchActiveQuality(qualities.get(qualities.size() - 1).getText());
    }

    private void loadFavoriteAndWatched() {
        switchFavorite(favoriteService.isFavorite(media));
        switchWatched(watchedService.isWatched(media));
    }

    private void loadSubtitles(SubtitlesRetrievedActivity activity) {
        if (!activity.getImdbId().equals(media.getImdbId()))
            return;

        // filter out all the subtitles that don't have a flag
        final List<Subtitle> subtitles = activity.getSubtitles().stream()
                .filter(e -> e.getFlagResource().isPresent())
                .sorted()
                .collect(Collectors.toList());

        Platform.runLater(() -> {
            languageSelection.getItems().clear();
            languageSelection.getItems().addAll(subtitles);
            languageSelection.select(0);
        });
    }

    private String getHealthTooltip(Torrent torrent, TorrentHealth health) {
        return localeText.get(health.getStatus().getKey()) + " - Ratio: " + String.format("%1$,.2f", health.getRatio()) + "\n" +
                "Seeds: " + torrent.getSeed() + " - Peers: " + torrent.getPeer();
    }

    private Label createQualityOption(String quality) {
        Label label = new Label(quality);

        label.getStyleClass().add("quality");
        label.setOnMouseClicked(this::onQualityClicked);

        return label;
    }

    private void openMagnetLink(Torrent torrent) {
        try {
            application.getHostServices().showDocument(torrent.getUrl());
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    private void copyMagnetLink(Torrent torrent) {
        ClipboardContent clipboardContent = new ClipboardContent();
        clipboardContent.putUrl(torrent.getUrl());
        clipboardContent.putString(torrent.getUrl());
        Clipboard.getSystemClipboard().setContent(clipboardContent);
    }

    private void switchHealth() {
        health.getStyleClass().removeIf(e -> !e.equals("health"));

        Torrent torrent = media.getTorrents().get(DEFAULT_TORRENT_AUDIO).get(quality);

        TorrentHealth health = torrentService.calculateHealth(torrent.getSeed(), torrent.getPeer());

        this.health.getStyleClass().add(health.getStatus().getStyleClass());
        Tooltip healthTooltip = new Tooltip(getHealthTooltip(torrent, health));
        healthTooltip.setWrapText(true);
        setInstantTooltip(healthTooltip);
        Tooltip.install(this.health, healthTooltip);
    }

    private void switchActiveQuality(String quality) {
        String activeStyle = "active";

        this.quality = quality;

        qualitySelectionPane.getChildren().forEach(e -> e.getStyleClass().remove(activeStyle));
        qualitySelectionPane.getChildren().stream()
                .map(e -> (Label) e)
                .filter(e -> e.getText().equalsIgnoreCase(quality))
                .findFirst()
                .ifPresent(e -> e.getStyleClass().add(activeStyle));

        switchHealth();
    }

    private void switchFavorite(boolean isLiked) {
        this.liked = isLiked;

        if (isLiked) {
            favoriteIcon.getStyleClass().add(LIKED_STYLE_CLASS);
            favoriteText.setText(localeText.get(DetailsMessage.REMOVE_FROM_BOOKMARKS));
        } else {
            favoriteIcon.getStyleClass().remove(LIKED_STYLE_CLASS);
            favoriteText.setText(localeText.get(DetailsMessage.ADD_TO_BOOKMARKS));
        }
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

    private void onQualityClicked(MouseEvent event) {
        Label label = (Label) event.getSource();

        switchActiveQuality(label.getText());
    }

    @FXML
    private void onMagnetClicked(MouseEvent event) {
        Optional<Torrent> torrent = media.getTorrents().values().stream()
                .findFirst()
                .map(e -> e.values().stream())
                .flatMap(Stream::findFirst);

        if (event.getButton() == MouseButton.SECONDARY) {
            torrent.ifPresent(this::copyMagnetLink);
        } else {
            torrent.ifPresent(this::openMagnetLink);
        }
    }

    @FXML
    private void onWatchNowClicked() {
        activityManager.register(new LoadMovieActivity() {
            @Override
            public String getQuality() {
                return quality;
            }

            @Override
            public Media getMedia() {
                return media;
            }

            @Override
            public Optional<Torrent> getTorrent() {
                return Optional.of(media.getTorrents().get(DEFAULT_TORRENT_AUDIO).get(quality));
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
            public Media getMedia() {
                return media;
            }

            @Override
            public Optional<Torrent> getTorrent() {
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
}
