package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.Rating;
import com.github.yoep.popcorn.ui.view.controls.Stars;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.beans.value.ChangeListener;
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
import java.util.ResourceBundle;

import static java.util.Arrays.asList;

@Slf4j
public class OverlayMediaCardComponent extends AbstractMediaCardComponent implements Initializable {
    private static final String LIKED_STYLE_CLASS = "liked";
    private static final String WATCHED_STYLE_CLASS = "watched";

    private final List<OverlayItemListener> listeners = new ArrayList<>();
    private final ChangeListener<Boolean> watchedListener = (observable, oldValue, newValue) -> switchWatched(newValue);
    private final OverlayItemMetadataProvider metadataProvider;

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
    }

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        super.initialize(location, resources);
        initializeRating();
        initializeStars();
        initializeFavorite();
        initializeWatched();
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

    private void initializeFavorite() {
        switchFavorite(metadataProvider.isLiked(media));
    }

    private void initializeWatched() {
        switchWatched(media.isWatched());
        media.watchedProperty().addListener(watchedListener);
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

    @FXML
    private void onWatchedClicked(MouseEvent event) {
        event.consume();
        boolean newValue = !media.isWatched();

        synchronized (listeners) {
            listeners.forEach(e -> e.onWatchedChanged(media, newValue));
        }
    }

    @FXML
    private void onFavoriteClicked(MouseEvent event) {
        event.consume();
        boolean newState = !metadataProvider.isLiked(media);
        metadataProvider.updateLikedState(media, newState);

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
