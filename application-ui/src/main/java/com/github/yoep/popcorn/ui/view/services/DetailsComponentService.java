package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.media.favorites.FavoriteEventCallback;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.media.watched.WatchedEventCallback;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.view.listeners.DetailsComponentListener;
import javafx.application.Platform;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class DetailsComponentService extends AbstractListenerService<DetailsComponentListener> {
    private final FavoriteService favoriteService;
    private final WatchedService watchedService;
    private final ApplicationConfig applicationConfig;
    private final SubtitlePickerService subtitlePickerService;

    private final FavoriteEventCallback favoriteEventCallback = createFavoriteCallback();
    private final WatchedEventCallback watchedEventCallback = createWatchedCallback();

    public DetailsComponentService(FavoriteService favoriteService, WatchedService watchedService, ApplicationConfig applicationConfig, SubtitlePickerService subtitlePickerService) {
        this.favoriteService = favoriteService;
        this.watchedService = watchedService;
        this.applicationConfig = applicationConfig;
        this.subtitlePickerService = subtitlePickerService;
        init();
    }

    public CompletableFuture<Boolean> isWatched(Media media) {
        return watchedService.isWatched(media);
    }

    public boolean isLiked(Media media) {
        return favoriteService.isLiked(media);
    }

    public boolean isTvMode() {
        return applicationConfig.isTvMode();
    }

    public void updateWatchedStated(Media media, boolean isWatched) {
        Objects.requireNonNull(media, "media cannot be null");
        if (isWatched) {
            watchedService.addToWatchList(media);
        } else {
            watchedService.removeFromWatchList(media);
        }
    }

    public void toggleWatchedState(Media media) {
        Objects.requireNonNull(media, "media cannot be null");
        watchedService.isWatched(media).whenComplete((watched, throwable) -> {
            if (throwable == null) {
                updateWatchedStated(media, !watched);
            } else {
                log.error("Failed to retrieve is watched", throwable);
            }
        });

    }

    public void toggleLikedState(Media media) {
        Objects.requireNonNull(media, "media cannot be null");
        if (favoriteService.isLiked(media)) {
            favoriteService.removeFromFavorites(media);
        } else {
            favoriteService.addToFavorites(media);
        }
    }

    public void onCustomSubtitleSelected(Runnable onCancelled) {
        Platform.runLater(() -> {
            var subtitleInfo = subtitlePickerService.pickCustomSubtitle();

            // if a custom subtitle was picked by the user, update the subtitle with the custom subtitle
            // otherwise, the subtitle pick was cancelled and we need to reset the selected language to disabled
            subtitleInfo.ifPresentOrElse(
                    e -> {
                    },
                    onCancelled::run);
        });
    }

    private void init() {
//        favoriteService.registerListener(favoriteEventCallback);
//        watchedService.registerListener(watchedEventCallback);
    }

    private FavoriteEventCallback createFavoriteCallback() {
        return event -> {
            switch (event.tag) {
                case LikedStateChanged -> {
                    var stateChanged = event.getUnion().getLiked_state_changed();
                    invokeListeners(e -> e.onLikedChanged(stateChanged.getImdbId(), stateChanged.getNewState()));
                }
            }
        };
    }

    private WatchedEventCallback createWatchedCallback() {
        return event -> {
            switch (event.tag) {
                case WatchedStateChanged -> {
                    var stateChanged = event.getUnion().getWatched_state_changed();
                    invokeListeners(e -> e.onWatchChanged(stateChanged.getImdbId(), stateChanged.getNewState()));
                }
            }
        };
    }
}
