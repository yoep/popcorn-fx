package com.github.yoep.popcorn.backend.media.watched;

import com.github.yoep.popcorn.backend.events.PlayerStoppedEvent;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.MediaType;
import com.github.yoep.popcorn.backend.media.watched.models.Watchable;
import com.github.yoep.popcorn.backend.media.watched.models.Watched;
import com.github.yoep.popcorn.backend.storage.StorageException;
import com.github.yoep.popcorn.backend.storage.StorageService;
import com.github.yoep.popcorn.backend.utils.IdleTimer;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.event.EventListener;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;
import java.time.Duration;
import java.util.ArrayList;
import java.util.List;

/**
 * The watched service maintains all the watched {@link Media} items of the application.
 * This is done through the {@link Watchable} items that are received from events and marking them as watched in the {@link #STORAGE_NAME} file.
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class WatchedService {
    static final String STORAGE_NAME = "watched.json";
    private static final int WATCHED_PERCENTAGE_THRESHOLD = 85;
    private static final int IDLE_TIME = 10;

    private final IdleTimer idleTimer = new IdleTimer(Duration.ofSeconds(IDLE_TIME));
    private final StorageService storageService;
    private final Object cacheLock = new Object();

    /**
     * The currently loaded watched cache.
     * This cache is saved and unloaded after {@link #IDLE_TIME} seconds to free up memory.
     */
    private Watched cache;
    private int cacheHash;

    //region Methods

    /**
     * Check if the given watchable has been watched already.
     *
     * @param watchable The watchable to check the watched state for.
     * @return Returns true if the watchable has already been watched, else false.
     */
    public boolean isWatched(Watchable watchable) {
        Assert.notNull(watchable, "watchable cannot be null");
        String key = watchable.getId();

        return isWatched(key);
    }

    /**
     * Get the watched movie items.
     *
     * @return Returns a list of movie ID's that have been watched.
     */
    public List<String> getWatchedMovies() {
        loadWatchedFileToCache();
        List<String> movies;

        synchronized (cacheLock) {
            movies = new ArrayList<>(cache.getMovies());
        }

        return movies;
    }

    /**
     * Get the watched show items.
     *
     * @return Returns a list of show ID's that have been watched.
     */
    public List<String> getWatchedShows() {
        loadWatchedFileToCache();
        List<String> movies;

        synchronized (cacheLock) {
            movies = new ArrayList<>(cache.getShows());
        }

        return movies;
    }

    /**
     * Add the watchable item to the watched list.
     *
     * @param watchable the watchable item to add.
     */
    public void addToWatchList(Watchable watchable) {
        Assert.notNull(watchable, "watchable cannot be null");
        String key = watchable.getId();

        addToWatchList(key, watchable.getType());
        watchable.setWatched(true);
    }

    /**
     * Remove the watchable item from the watched list.
     *
     * @param watchable The watchable item to remove.
     */
    public void removeFromWatchList(Watchable watchable) {
        Assert.notNull(watchable, "watchable cannot be null");
        String key = watchable.getId();
        loadWatchedFileToCache();

        synchronized (cacheLock) {
            cache.remove(key);
            watchable.setWatched(false);
        }
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

    //region PostConstruct

    @PostConstruct
    void init() {
        initializeIdleTimer();
    }

    private void initializeIdleTimer() {
        idleTimer.setOnTimeout(this::onSave);
    }

    //endregion

    //region PreDestroy

    @PreDestroy
    void destroy() {
        onSave();
    }

    //endregion

    //region Functions

    private void addToWatchList(String key, MediaType type) {
        loadWatchedFileToCache();

        synchronized (cacheLock) {
            // prevent keys from being added twice
            if (cache.contains(key))
                return;

            if (type == MediaType.MOVIE) {
                cache.addMovie(key);
            } else {
                cache.addShow(key);
            }
        }
    }

    private boolean isWatched(String key) {
        loadWatchedFileToCache();

        synchronized (cacheLock) {
            return cache.contains(key);
        }
    }

    private void save(Watched watched) {
        try {
            log.debug("Saving watched items to storage");
            storageService.store(STORAGE_NAME, watched);
        } catch (StorageException ex) {
            log.error("Failed to save the watched items with error " + ex.getMessage(), ex);
        }
    }

    private void loadWatchedFileToCache() {
        idleTimer.runFromStart();

        // check if cache is still present
        // if so, return the cache
        if (cache != null) {
            log.trace("Not updating cache as it's already present");
            return;
        }

        log.debug("Loading watched items from storage");
        try {
            storageService.read(STORAGE_NAME, Watched.class)
                    .ifPresentOrElse(this::handleStoredWatchedItems, this::createNewWatchedItems);
        } catch (StorageException ex) {
            log.error("Failed to read watched items, {}", ex.getMessage(), ex);
        }
    }

    private void handleStoredWatchedItems(Watched e) {
        synchronized (cacheLock) {
            cache = e;
            cacheHash = cache.hashCode();
        }
    }

    private void createNewWatchedItems() {
        synchronized (cacheLock) {
            cache = Watched.builder().build();
        }
    }

    private void onSave() {
        if (cache == null)
            return;

        synchronized (cacheLock) {
            // check if the cache was modified
            // if not, the cache will only be removed from memory but not saved again
            if (cache.hashCode() != cacheHash)
                save(cache);

            cache = null;
        }
    }

    //endregion
}