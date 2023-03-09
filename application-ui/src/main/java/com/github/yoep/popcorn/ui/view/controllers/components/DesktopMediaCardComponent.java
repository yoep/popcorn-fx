package com.github.yoep.popcorn.ui.view.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteEventCallback;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.Rating;
import com.github.yoep.popcorn.backend.media.providers.models.ShowOverview;
import com.github.yoep.popcorn.backend.media.watched.WatchedEventCallback;
import com.github.yoep.popcorn.ui.messages.MediaMessage;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.OverlayItemListener;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.OverlayItemMetadataProvider;
import com.github.yoep.popcorn.ui.view.controls.Stars;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.fxml.FXML;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Objects;
import java.util.ResourceBundle;

@Slf4j
public class DesktopMediaCardComponent extends TvMediaCardComponent {
    private static final String LIKED_STYLE_CLASS = "liked";
    private static final String WATCHED_STYLE_CLASS = "watched";

    private final LocaleText localeText;
    private final OverlayItemMetadataProvider metadataProvider;
    private final FavoriteEventCallback favoriteEventCallback = createFavoriteCallback(media);
    private final WatchedEventCallback watchedEventCallback = createWatchedCallback(media);

    @FXML
    Pane posterItem;
    @FXML
    Label ratingValue;
    @FXML
    Icon favorite;
    @FXML
    Stars ratingStars;
    @FXML
    Label title;
    @FXML
    Label year;
    @FXML
    Label seasons;

    public DesktopMediaCardComponent(Media media,
                                     LocaleText localeText,
                                     ImageService imageService,
                                     OverlayItemMetadataProvider metadataProvider,
                                     OverlayItemListener... listeners) {
        super(media, imageService, listeners);
        this.localeText = localeText;
        this.metadataProvider = metadataProvider;

        metadataProvider.addListener(favoriteEventCallback);
        metadataProvider.addListener(watchedEventCallback);
    }

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        super.initialize(location, resources);
        initializeText();
        initializeRating();
        initializeStars();
        initializeMetadata();
        initializeParentListener();
    }

    private void initializeText() {
        title.setText(media.getTitle());
        year.setText(media.getYear());

        if (media instanceof ShowOverview) {
            var show = (ShowOverview) media;
            var text = localeText.get(MediaMessage.SEASONS, show.getNumberOfSeasons());

            if (show.getNumberOfSeasons() > 1) {
                text += localeText.get(MediaMessage.PLURAL);
            }

            seasons.setText(text);
        }

        Tooltip.install(title, new Tooltip(media.getTitle()));
    }

    private void initializeRating() {
        media.getRating()
                .map(Rating::getPercentage)
                .map(e -> (double) e / 10)
                .map(e -> e + "/10")
                .ifPresent(ratingValue::setText);
    }

    private void initializeStars() {
        media.getRating().ifPresent(ratingStars::setRating);
    }

    private void initializeMetadata() {
        switchFavorite(metadataProvider.isLiked(media));
        switchWatched(metadataProvider.isWatched(media));
    }

    private void initializeParentListener() {
        posterItem.parentProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue == null) {
                metadataProvider.removeListener(favoriteEventCallback);
                metadataProvider.removeListener(watchedEventCallback);
            }
        });
    }

    private void switchFavorite(boolean isFavorite) {
        if (isFavorite) {
            favorite.getStyleClass().add(LIKED_STYLE_CLASS);
        } else {
            favorite.getStyleClass().remove(LIKED_STYLE_CLASS);
        }
    }

    private void switchWatched(boolean isWatched) {
        if (isWatched) {
            posterItem.getStyleClass().add(WATCHED_STYLE_CLASS);
        } else {
            posterItem.getStyleClass().remove(WATCHED_STYLE_CLASS);
        }
    }

    private FavoriteEventCallback createFavoriteCallback(Media media) {
        return event -> {
            switch (event.getTag()) {
                case LikedStateChanged -> {
                    var stateChange = event.getUnion().getLiked_state_changed();

                    if (Objects.equals(stateChange.getImdbId(), media.getId())) {
                        switchFavorite(stateChange.getNewState());
                    }
                }
            }
        };
    }

    private WatchedEventCallback createWatchedCallback(Media media) {
        return event -> {
            switch (event.getTag()) {
                case WatchedStateChanged -> {
                    var stateChange = event.getUnion().getWatched_state_changed();

                    if (Objects.equals(stateChange.getImdbId(), media.getId())) {
                        switchWatched(stateChange.getNewState());
                    }
                }
            }
        };
    }

    @FXML
    void onWatchedClicked(MouseEvent event) {
        event.consume();
        boolean newValue = !metadataProvider.isWatched(media);

        synchronized (listeners) {
            listeners.forEach(e -> e.onWatchedChanged(media, newValue));
        }
    }

    @FXML
    void onFavoriteClicked(MouseEvent event) {
        event.consume();
        boolean newState = !metadataProvider.isLiked(media);

        synchronized (listeners) {
            listeners.forEach(e -> e.onFavoriteChanged(media, newState));
        }
    }
}
