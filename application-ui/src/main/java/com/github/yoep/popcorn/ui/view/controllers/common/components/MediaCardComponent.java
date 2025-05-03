package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.FavoriteEvent;
import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.media.ShowOverview;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteEventListener;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.font.controls.Icon;
import com.github.yoep.popcorn.ui.messages.MediaMessage;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.OverlayItemListener;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.OverlayItemMetadataProvider;
import com.github.yoep.popcorn.ui.view.controls.Stars;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.Parent;
import javafx.scene.control.Label;
import javafx.scene.control.Tooltip;
import javafx.scene.input.MouseEvent;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.Objects;
import java.util.ResourceBundle;

import static com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Rating;

@Slf4j
public class MediaCardComponent extends TvMediaCardComponent implements FavoriteEventListener {
    private static final String LIKED_STYLE_CLASS = "liked";

    private final LocaleText localeText;

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

    public MediaCardComponent(Media media,
                              LocaleText localeText,
                              ImageService imageService,
                              OverlayItemMetadataProvider metadataProvider,
                              OverlayItemListener... listeners) {
        super(media, imageService, metadataProvider, listeners);
        this.localeText = localeText;

        metadataProvider.addFavoriteListener(this);
    }

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        super.initialize(location, resources);
        initializeText();
        initializeRating();
        initializeStars();
        initializeMetadata();
    }

    @Override
    public void onLikedStateChanged(FavoriteEvent.LikedStateChanged event) {
        if (Objects.equals(event.getImdbId(), media.id())) {
            switchFavorite(event.getIsLiked());
        }
    }

    @Override
    protected void initializeMetadata() {
        super.initializeMetadata();
        metadataProvider.isLiked(media).whenComplete((isLiked, throwable) -> {
            if (throwable == null) {
                switchFavorite(isLiked);
            } else {
                log.error("Failed to retrieve is liked", throwable);
            }
        });
    }

    @Override
    protected void onParentChanged(Parent newValue) {
        super.onParentChanged(newValue);
        if (newValue == null) {
            metadataProvider.removeFavoriteListener(this);
        }
    }

    private void initializeText() {
        title.setText(media.title());
        year.setText(media.year());

        if (media instanceof ShowOverview) {
            var show = (ShowOverview) media;
            var text = localeText.get(MediaMessage.SEASONS, show.getSeasons());

            if (show.getSeasons() > 1) {
                text += localeText.get(MediaMessage.PLURAL);
            }

            seasons.setText(text);
        }

        Tooltip.install(title, new Tooltip(media.title()));
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

    private void switchFavorite(boolean isFavorite) {
        Platform.runLater(() -> {
            if (isFavorite) {
                favorite.getStyleClass().add(LIKED_STYLE_CLASS);
            } else {
                favorite.getStyleClass().remove(LIKED_STYLE_CLASS);
            }
        });
    }

    @FXML
    void onWatchedClicked(MouseEvent event) {
        event.consume();
        metadataProvider.isWatched(media)
                .thenAccept(isWatched -> listeners.forEach(e -> e.onWatchedChanged(media, !isWatched)));
    }

    @FXML
    void onFavoriteClicked(MouseEvent event) {
        event.consume();
        metadataProvider.isLiked(media)
                .thenAccept(isLiked -> listeners.forEach(e -> e.onFavoriteChanged(media, !isLiked)));
    }
}
