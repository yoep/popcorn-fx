package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteEvent;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.watched.WatchedEvent;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.control.Tooltip;
import javafx.scene.input.MouseEvent;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;
import java.util.Objects;

@Slf4j
public class DesktopPosterComponent extends TvPosterComponent {
    static final String LIKED_STYLE_CLASS = "liked";
    static final String WATCHED_STYLE_CLASS = "seen";

    private final FavoriteService favoriteService;
    private final WatchedService watchedService;
    private final LocaleText localeText;

    public DesktopPosterComponent(EventPublisher eventPublisher, ImageService imageService, FavoriteService favoriteService, WatchedService watchedService,
                                  LocaleText localeText) {
        super(eventPublisher, imageService);
        this.favoriteService = favoriteService;
        this.watchedService = watchedService;
        this.localeText = localeText;
    }

    @FXML
    Icon watchedIcon;
    @FXML
    Tooltip watchedTooltip;
    @FXML
    Icon favoriteIcon;
    @FXML
    Tooltip favoriteTooltip;

    @PostConstruct
    @Override
    void init() {
        super.init();
        watchedService.registerListener(event -> {
            if (event.getTag() == WatchedEvent.Tag.WatchedStateChanged) {
                var stateChanged = event.getUnion().getWatched_state_changed();
                if (Objects.equals(stateChanged.getImdbId(), media.getId())) {
                    updateWatchedState(stateChanged.getNewState());
                }
            }
        });
        favoriteService.removeListener(event -> {
            if (event.getTag() == FavoriteEvent.Tag.LikedStateChanged) {
                var stateChanged = event.getUnion().getLiked_state_changed();
                if (Objects.equals(stateChanged.getImdbId(), media.getId())) {
                    updateLikedState(stateChanged.getNewState());
                }
            }
        });
    }

    @Override
    void onPlayEvent(Media media) {
        super.onPlayEvent(media);
        updateWatchedState(watchedService.isWatched(media));
        updateLikedState(favoriteService.isLiked(media));
    }

    private void toggleLikedState() {
        if (favoriteService.isLiked(media)) {
            favoriteService.removeFromFavorites(media);
        } else {
            favoriteService.addToFavorites(media);
        }
    }

    private void toggleWatchedState() {
        if (watchedService.isWatched(media)) {
            watchedService.removeFromWatchList(media);
        } else {
            watchedService.addToWatchList(media);
        }
    }

    private void updateLikedState(boolean newState) {
        Platform.runLater(() -> {
            if (newState) {
                favoriteIcon.setText(Icon.HEART_UNICODE);
                favoriteTooltip.setText(localeText.get(DetailsMessage.REMOVE_FROM_BOOKMARKS));
                favoriteIcon.getStyleClass().add(LIKED_STYLE_CLASS);
            } else {
                favoriteIcon.setText(Icon.HEART_O_UNICODE);
                favoriteTooltip.setText(localeText.get(DetailsMessage.ADD_TO_BOOKMARKS));
                favoriteIcon.getStyleClass().removeIf(e -> e.equals(LIKED_STYLE_CLASS));
            }
        });
    }

    private void updateWatchedState(boolean newState) {
        Platform.runLater(() -> {
            if (newState) {
                watchedIcon.setText(Icon.CHECK_UNICODE);
                watchedTooltip.setText(localeText.get(DetailsMessage.MARK_AS_NOT_SEEN));
                watchedIcon.getStyleClass().add(WATCHED_STYLE_CLASS);
            } else {
                watchedIcon.setText(Icon.EYE_SLASH_UNICODE);
                watchedTooltip.setText(localeText.get(DetailsMessage.MARK_AS_SEEN));
                watchedIcon.getStyleClass().removeIf(e -> e.equals(WATCHED_STYLE_CLASS));
            }
        });
    }

    @FXML
    void onFavoriteClicked(MouseEvent event) {
        event.consume();
        toggleLikedState();
    }

    @FXML
    void onWatchedClicked(MouseEvent event) {
        event.consume();
        toggleWatchedState();
    }
}
