package com.github.yoep.popcorn.backend.media.watched;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import com.github.yoep.popcorn.backend.events.PlayerStoppedEvent;
import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.watched.models.Watchable;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import java.util.List;
import java.util.concurrent.ConcurrentLinkedDeque;

/**
 * The watched service maintains all the watched {@link Media} items of the application.
 * This is done through the {@link Watchable} items that are received from events and marking them as watched.
 */
@Slf4j
@Service
public class WatchedService {
    private static final int WATCHED_PERCENTAGE_THRESHOLD = 85;
    private final Object lock = new Object();
    private final WatchedEventCallback callback = createCallback();
    private final ConcurrentLinkedDeque<WatchedEventCallback> listeners = new ConcurrentLinkedDeque<>();

    public WatchedService() {
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
        Assert.notNull(watchable, "watchable cannot be null");
        synchronized (lock) {
            return FxLib.INSTANCE.is_media_watched(PopcornFxInstance.INSTANCE.get(), MediaItem.from(watchable));
        }
    }

    /**
     * Get the watched movie items.
     *
     * @return Returns a list of movie ID's that have been watched.
     */
    public List<String> getWatchedMovies() {
        synchronized (lock) {
            try (var watched = FxLib.INSTANCE.retrieve_watched_movies(PopcornFxInstance.INSTANCE.get())) {
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
            try (var watched = FxLib.INSTANCE.retrieve_watched_shows(PopcornFxInstance.INSTANCE.get())) {
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
        Assert.notNull(watchable, "watchable cannot be null");
        synchronized (lock) {
            FxLib.INSTANCE.add_to_watched(PopcornFxInstance.INSTANCE.get(), MediaItem.from(watchable));
        }
    }

    /**
     * Remove the watchable item from the watched list.
     *
     * @param watchable The watchable item to remove.
     */
    public void removeFromWatchList(Media watchable) {
        Assert.notNull(watchable, "watchable cannot be null");
        synchronized (lock) {
            FxLib.INSTANCE.remove_from_watched(PopcornFxInstance.INSTANCE.get(), MediaItem.from(watchable));
        }
    }

    public void registerListener(WatchedEventCallback callback) {
        Assert.notNull(callback, "callback cannot be null");
        listeners.add(callback);
    }

    public void removeListener(WatchedEventCallback callback) {
        listeners.remove(callback);
    }

    @EventListener
    public void onPlayerStopped(PlayerStoppedEvent event) {
        // check if the media is present
        // if not, the played video might have been a trailer or video file
        if (event.getMedia().isEmpty())
            return;

        var time = event.getTime();
        var duration = event.getDuration();

        // check if both the time and duration of the video are known
        // if not, the close activity media is not eligible for being auto marked as watched
        if (time == PlayerStoppedEvent.UNKNOWN || duration == PlayerStoppedEvent.UNKNOWN)
            return;

        var percentageWatched = ((double) time / duration) * 100;
        var media = event.getMedia().get();

        // check if the media has been watched for the percentage threshold
        // if so, mark the media as watched
        log.trace("Media playback of \"{}\" ({}) has been watched for {}%", media.getTitle(), media.getId(), percentageWatched);
        if (percentageWatched >= WATCHED_PERCENTAGE_THRESHOLD) {
            log.debug("Marking media \"{}\" ({}) automatically as watched", media.getTitle(), media.getId());
            addToWatchList(media);
        }
    }

    //endregion

    private void init() {
        synchronized (lock) {
            FxLib.INSTANCE.register_watched_event_callback(PopcornFxInstance.INSTANCE.get(), callback);
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