package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.events.ShowDetailsEvent;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.events.ShowSerieDetailsEvent;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteEventCallback;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.watched.WatchedEventCallback;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.ui.view.listeners.DetailsComponentListener;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.util.Objects;

@Slf4j
@Service
@RequiredArgsConstructor
public class DetailsComponentService extends AbstractListenerService<DetailsComponentListener> {
    private final FavoriteService favoriteService;
    private final WatchedService watchedService;

    private final FavoriteEventCallback favoriteEventCallback = createFavoriteCallback();
    private final WatchedEventCallback watchedEventCallback = createWatchedCallback();

    private Media lastShownMediaItem;

    @EventListener
    public void onShowDetails(ShowDetailsEvent event) {
        if (event instanceof ShowMovieDetailsEvent movieEvent) {
            lastShownMediaItem = movieEvent.getMedia();
        } else if (event instanceof ShowSerieDetailsEvent serieEvent) {
            lastShownMediaItem = serieEvent.getMedia();
        } else {
            log.warn("Unknown details events received, event: {}", event.getClass().getSimpleName());
        }
    }

    public boolean isWatched(Media media) {
        return watchedService.isWatched(media);
    }

    public boolean isLiked(Media media) {
        return favoriteService.isLiked(media);
    }

    public void updateWatchedStated(Media media, boolean isWatched) {
        Objects.requireNonNull(media, "media cannot be null");
        if (isWatched) {
            watchedService.addToWatchList(media);
        } else {
            watchedService.removeFromWatchList(media);
        }
    }

    public void toggleWatchedState() {
        if (lastShownMediaItem == null) {
            log.warn("Unable to update watch state, media item is unknown");
            return;
        }

        updateWatchedStated(lastShownMediaItem, !watchedService.isWatched(lastShownMediaItem));
    }

    public void toggleLikedState() {
        if (lastShownMediaItem == null) {
            log.warn("Unable to update liked state, media item is unknown");
            return;
        }

        if (favoriteService.isLiked(lastShownMediaItem)) {
            favoriteService.removeFromFavorites(lastShownMediaItem);
        } else {
            favoriteService.addToFavorites(lastShownMediaItem);
        }
    }

    @PostConstruct
    private void init() {
        favoriteService.registerListener(favoriteEventCallback);
        watchedService.registerListener(watchedEventCallback);
    }

    private FavoriteEventCallback createFavoriteCallback() {
        return event -> {
            switch (event.tag) {
                case LikedStateChanged -> {
                    var stateChanged = event.getUnion().getLiked_state_changed();
                    invokeListeners(e -> e.onLikedChanged(stateChanged.getNewState()));
                }
            }
        };
    }

    private WatchedEventCallback createWatchedCallback() {
        return event -> {
            switch (event.tag) {
                case WatchedStateChanged -> {
                    var stateChanged = event.getUnion().getWatched_state_changed();
                    invokeListeners(e -> e.onWatchChanged(stateChanged.getNewState()));
                }
            }
        };
    }
}
