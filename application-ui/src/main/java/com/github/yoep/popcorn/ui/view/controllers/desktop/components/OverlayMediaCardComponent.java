package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteEventCallback;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.Rating;
import com.github.yoep.popcorn.backend.media.watched.WatchedEventCallback;
import com.github.yoep.popcorn.ui.view.controls.Stars;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.net.URL;
import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.ResourceBundle;

import static java.util.Arrays.asList;

@Slf4j
public class OverlayMediaCardComponent extends AbstractMediaCardComponent implements Initializable {
    private static final String LIKED_STYLE_CLASS = "liked";
    private static final String WATCHED_STYLE_CLASS = "watched";

    private final List<OverlayItemListener> listeners = new ArrayList<>();
    private final OverlayItemMetadataProvider metadataProvider;
    private final FavoriteEventCallback favoriteEventCallback = createFavoriteCallback(media);
    private final WatchedEventCallback watchedEventCallback = createWatchedCallback(media);

    @FXML
    private Pane posterItem;
    @FXML
    private Label ratingValue;
    @FXML
    private Icon favorite;
    @FXML
    private Stars ratingStars;

    public OverlayMediaCardComponent(Media media,
                                     LocaleText localeText,
                                     ImageService imageService,
                                     OverlayItemMetadataProvider metadataProvider,
                                     OverlayItemListener... listeners) {
        super(media, localeText, imageService);
        this.metadataProvider = metadataProvider;
        this.listeners.addAll(asList(listeners));

        metadataProvider.addListener(favoriteEventCallback);
        metadataProvider.addListener(watchedEventCallback);
    }

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        super.initialize(location, resources);
        initializeRating();
        initializeStars();
        initializeMetadata();
        initializeParentListener();
    }

    /**
     * Add a listener to this instance.
     *
     * @param listener The listener to add.
     */
    public void addListener(OverlayItemListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.add(listener);
        }
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
    private void onWatchedClicked(MouseEvent event) {
        event.consume();
        boolean newValue = !metadataProvider.isWatched(media);

        synchronized (listeners) {
            listeners.forEach(e -> e.onWatchedChanged(media, newValue));
        }
    }

    @FXML
    private void onFavoriteClicked(MouseEvent event) {
        event.consume();
        boolean newState = !metadataProvider.isLiked(media);

        synchronized (listeners) {
            listeners.forEach(e -> e.onFavoriteChanged(media, newState));
        }
    }

    @FXML
    private void showDetails() {
        synchronized (listeners) {
            listeners.forEach(e -> e.onClicked(media));
        }
    }
}
