package com.github.yoep.popcorn.backend.media.watched;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.watched.models.Watchable;
import lombok.extern.slf4j.Slf4j;

import java.util.List;
import java.util.Objects;
import java.util.concurrent.ConcurrentLinkedDeque;

/**
 * The watched service maintains all the watched {@link Media} items of the application.
 * This is done through the {@link Watchable} items that are received from events and marking them as watched.
 */
@Slf4j
public class WatchedService {
    private final FxLib fxLib;
    private final PopcornFx instance;

    private final Object lock = new Object();
    private final WatchedEventCallback callback = createCallback();
    private final ConcurrentLinkedDeque<WatchedEventCallback> listeners = new ConcurrentLinkedDeque<>();

    public WatchedService(FxLib fxLib, PopcornFx instance) {
        this.fxLib = fxLib;
        this.instance = instance;
        init();
    }

    //region Methods

    /**
     * Check if the given watchable has been watched already.
     *
     * @param watchable The watchable to check the watched state for.
     * @return Returns true if the watchable has already been watched, else false.
     */
    public boolean isWatched(Media watchable) {
        Objects.requireNonNull(watchable, "watchable cannot be null");
        synchronized (lock) {
            try (var media = MediaItem.from(watchable)) {
                return fxLib.is_media_watched(instance, media) == 1;
            }
        }
    }

    /**
     * Get the watched movie items.
     *
     * @return Returns a list of movie ID's that have been watched.
     */
    public List<String> getWatchedMovies() {
        synchronized (lock) {
            try (var watched = fxLib.retrieve_watched_movies(instance)) {
                log.debug("Retrieved watched movies {}", watched);
                return watched.values();
            }
        }
    }

    /**
     * Get the watched show items.
     *
     * @return Returns a list of show ID's that have been watched.
     */
    public List<String> getWatchedShows() {
        synchronized (lock) {
            try (var watched = fxLib.retrieve_watched_shows(instance)) {
                log.debug("Retrieved watched shows {}", watched);
                return watched.values();
            }
        }
    }

    /**
     * Add the watchable item to the watched list.
     *
     * @param watchable the watchable item to add.
     */
    public void addToWatchList(Media watchable) {
        Objects.requireNonNull(watchable, "watchable cannot be null");
        synchronized (lock) {
            try (var media = MediaItem.from(watchable)) {
                fxLib.add_to_watched(instance, media);
            }
        }
    }

    /**
     * Remove the watchable item from the watched list.
     *
     * @param watchable The watchable item to remove.
     */
    public void removeFromWatchList(Media watchable) {
        Objects.requireNonNull(watchable, "watchable cannot be null");
        synchronized (lock) {
            try (var media = MediaItem.from(watchable)) {
                fxLib.remove_from_watched(instance, media);
            }
        }
    }

    public void registerListener(WatchedEventCallback callback) {
        Objects.requireNonNull(callback, "callback cannot be null");
        listeners.add(callback);
    }

    public void removeListener(WatchedEventCallback callback) {
        listeners.remove(callback);
    }

    //endregion
    private void init() {
        synchronized (lock) {
            fxLib.register_watched_event_callback(instance, callback);
        }
    }

    private WatchedEventCallback createCallback() {
        return event -> {
            log.debug("Received watched event callback {}", event);

            try {
                for (var listener : listeners) {
                    listener.callback(event);
                }
            } catch (Exception ex) {
                log.error("Failed to invoke watched callback, {}", ex.getMessage(), ex);
            }
        };
    }
}
