package com.github.yoep.popcorn.backend.media.favorites;

import com.github.yoep.popcorn.backend.media.favorites.models.Favorable;
import com.github.yoep.popcorn.backend.media.favorites.models.Favorites;
import com.github.yoep.popcorn.backend.media.providers.ProviderService;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.ShowOverview;
import com.github.yoep.popcorn.backend.storage.StorageException;
import com.github.yoep.popcorn.backend.storage.StorageService;
import com.github.yoep.popcorn.backend.utils.IdleTimer;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;
import java.time.Duration;
import java.time.LocalDateTime;
import java.util.List;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;
import java.util.stream.Collectors;

@Slf4j
@Service
@RequiredArgsConstructor
public class FavoriteService {
    static final int UPDATE_CACHE_AFTER_HOURS = 72;
    static final String STORAGE_NAME = "favorites.json";
    private static final int IDLE_TIME = 10;

    private final IdleTimer idleTimer = new IdleTimer(Duration.ofSeconds(IDLE_TIME));
    private final TaskExecutor taskExecutor;
    private final StorageService storageService;
    private final ProviderService<Movie> movieProviderService;
    private final ProviderService<ShowOverview> showProviderService;
    private final Object cacheLock = new Object();

    /**
     * The currently loaded favorable cache.
     * This cache is saved and unloaded after {@link #IDLE_TIME} seconds to free up memory.
     */
    private Favorites cache;
    private int cacheHash;

    /**
     * Check if the given {@link Favorable} is liked by the user.
     *
     * @param favorable The favorable to check.
     * @return Returns true if the favorable is liked, else false.
     */
    public boolean isLiked(Favorable favorable) {
        Assert.notNull(favorable, "favorable cannot be null");
        loadFavorites();

        synchronized (cacheLock) {
            return cache.getAll().stream()
                    .filter(Objects::nonNull)
                    .anyMatch(e -> Objects.equals(e.getId(), favorable.getId()));
        }
    }

    /**
     * Get all the {@link Favorable} items that are liked by the user.
     *
     * @return Returns the list of liked items by the user.
     */
    public List<Favorable> getAll() {
        loadFavorites();

        synchronized (cacheLock) {
            return cache.getAll();
        }
    }

    /**
     * Add the given {@link Favorable} to the favorites.
     *
     * @param favorable The favorable to add.
     */
    public void addToFavorites(Favorable favorable) {
        Assert.notNull(favorable, "favorable cannot be null");
        loadFavorites();

        synchronized (cacheLock) {
            favorable.setLiked(true);

            // verify that the favorable doesn't already exist
            if (!isLiked(favorable)) {
                cache.add(favorable);
            }
        }
    }

    /**
     * Remove the given favorable from favorites.
     *
     * @param favorable The favorable to remove.
     */
    public void removeFromFavorites(Favorable favorable) {
        Assert.notNull(favorable, "favorable cannot be null");
        loadFavorites();

        synchronized (cacheLock) {
            cache.remove(favorable);
            favorable.setLiked(false);
        }
    }

    //region PostConstruct

    @PostConstruct
    void init() {
        initializeIdleTimer();
        initializeCacheRefresh();
    }

    private void initializeIdleTimer() {
        idleTimer.setOnTimeout(this::onSave);
    }

    private void initializeCacheRefresh() {
        try {
            if (isCacheUpdateRequired()) {
                // update the cache on a separate thread to not block the startup process
                log.trace("Favorites cache update is required, starting cache update thread");
                taskExecutor.execute(this::updateCache);
            }
        } catch (FavoriteException ex) {
            log.error("Failed to refresh favorites cache, " + ex.getMessage(), ex);
        }
    }

    //endregion

    //region PreDestroy

    @PreDestroy
    void destroy() {
        onSave();
    }

    //endregion

    //region Functions

    private void save(Favorites favorites) {
        try {
            storageService.store(STORAGE_NAME, favorites);
        } catch (StorageException ex) {
            log.error("Failed to save favorites, {}", ex.getMessage(), ex);
        }
    }

    private void loadFavorites() {
        idleTimer.runFromStart();

        // check if the cache has already been loaded
        // if so, do nothing
        synchronized (cacheLock) {
            if (cache != null)
                return;
        }

        log.debug("Loading favorites from storage");
        try {
            storageService.read(STORAGE_NAME, Favorites.class)
                    .ifPresentOrElse(this::handleStoredFavorites, this::createNewFavorites);
        } catch (StorageException ex) {
            throw new FavoriteException(ex.getMessage(), ex);
        }
    }

    private void handleStoredFavorites(Favorites e) {
        synchronized (cacheLock) {
            cache = e;
            cacheHash = cache.hashCode();
        }
    }

    private void createNewFavorites() {
        // build a new cache as now favorite database file is found
        log.debug("Favorites file was not found, creating a new blank cache for favorites");
        synchronized (cacheLock) {
            cache = Favorites.builder().build();
            cacheHash = cache.hashCode();
        }
    }

    private void updateCache() {
        log.debug("Starting favorites cache update");
        loadFavorites();

        idleTimer.stop();
        updateMoviesCache();
        updateSeriesCache();

        synchronized (cacheLock) {
            cache.setLastCacheUpdate(LocalDateTime.now());
        }

        idleTimer.runFromStart();
        log.info("Favorite cache has been updated");
    }

    private void updateMoviesCache() {
        log.trace("Updating movies favorite cache");
        var newMoviesCache = cache.getMovies().stream()
                .map(e -> movieProviderService.getDetails(e.getImdbId()))
                .map(CompletableFuture::join)
                .collect(Collectors.toList());

        synchronized (cacheLock) {
            cache.setMovies(newMoviesCache);
        }
    }

    private void updateSeriesCache() {
        log.trace("Updating shows favorite cache");
        var newShowsCache = cache.getShows().stream()
                .map(e -> showProviderService.getDetails(e.getImdbId()))
                .map(CompletableFuture::join)
                .collect(Collectors.toList());

        synchronized (cacheLock) {
            cache.setShows(newShowsCache);
        }
    }

    private boolean isCacheUpdateRequired() {
        var cacheUpdateDateTime = LocalDateTime.now().minusHours(UPDATE_CACHE_AFTER_HOURS);

        loadFavorites();

        boolean shouldUpdate;

        synchronized (cacheLock) {
            shouldUpdate = cache.getLastCacheUpdate() == null ||
                    cache.getLastCacheUpdate().isBefore(cacheUpdateDateTime);
        }

        return shouldUpdate;
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
