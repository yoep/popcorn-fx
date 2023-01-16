package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.events.ShowDetailsEvent;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.events.ShowSerieDetailsEvent;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteEventCallback;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.ui.view.listeners.DetailsComponentListener;
import javafx.beans.value.ChangeListener;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.util.Objects;
import java.util.Optional;

@Slf4j
@Service
@RequiredArgsConstructor
public class DetailsComponentService extends AbstractListenerService<DetailsComponentListener> {
    private final FavoriteService favoriteService;
    private final WatchedService watchedService;

    private final ChangeListener<Boolean> watchedListener = (observable, oldValue, newValue) -> onWatchedChanged(newValue);
    private final FavoriteEventCallback callback = createCallback();

    private Media lastShownMediaItem;

    @EventListener
    public void onShowDetails(ShowDetailsEvent event) {
        if (event instanceof ShowMovieDetailsEvent movieEvent) {
            var media = movieEvent.getMedia();
            subscribeToPropertyChanges(media);
            lastShownMediaItem = media;
        } else if (event instanceof ShowSerieDetailsEvent serieEvent) {
            var media = serieEvent.getMedia();
            subscribeToPropertyChanges(media);
            lastShownMediaItem = media;
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

        updateWatchedStated(lastShownMediaItem, !lastShownMediaItem.isWatched());
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
        favoriteService.registerListener(callback);
    }

    private void subscribeToPropertyChanges(Media media) {
        // remove the listeners from the old item
        // if one is present before registering to the new one
        Optional.ofNullable(lastShownMediaItem)
                .ifPresent(e -> {
                    e.watchedProperty().removeListener(watchedListener);
                });

        media.watchedProperty().addListener(watchedListener);
    }

    private void onWatchedChanged(boolean newValue) {
        invokeListeners(e -> e.onWatchChanged(newValue));
    }

    private FavoriteEventCallback createCallback() {
        return event -> {
            switch (event.tag) {
                case LikedStateChanged -> {
                    var stateChanged = event.getUnion().getLiked_state_changed();
                    invokeListeners(e -> e.onLikedChanged(stateChanged.getNewState()));
                }
            }
        };
    }
}
