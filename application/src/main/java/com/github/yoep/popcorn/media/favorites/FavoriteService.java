package com.github.yoep.popcorn.media.favorites;

import com.fasterxml.jackson.databind.ObjectMapper;
import com.github.yoep.popcorn.PopcornTimeApplication;
import com.github.yoep.popcorn.media.favorites.models.Favorable;
import com.github.yoep.popcorn.media.favorites.models.Favorites;
import javafx.animation.PauseTransition;
import javafx.util.Duration;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FileUtils;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;
import java.io.File;
import java.io.IOException;
import java.nio.charset.Charset;
import java.util.List;

@Slf4j
@Service
@RequiredArgsConstructor
public class FavoriteService {
    private static final int IDLE_TIME = 10;
    private static final String NAME = "favorites.json";

    private final PauseTransition idleTimer = new PauseTransition(Duration.seconds(IDLE_TIME));
    private final ObjectMapper objectMapper;
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
                    .anyMatch(e -> e.getId().equals(favorable.getId()));
        }
    }

    /**
     * Get the favorites.
     *
     * @return Returns the favorites.
     */
    public Favorites getFavorites() {
        loadFavorites();

        synchronized (cacheLock) {
            return cache;
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
            cache.add(favorable);
            favorable.setLiked(true);
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
    public void init() {
        initializeIdleTimer();
    }

    private void initializeIdleTimer() {
        idleTimer.setOnFinished(e -> onSave());
    }

    //endregion

    //region PreDestroy

    @PreDestroy
    private void destroy() {
        onSave();
    }

    //endregion

    //region Functions

    private void save(Favorites favorites) {
        File file = getFile();

        try {
            log.debug("Saving favorites to {}", file.getAbsolutePath());
            FileUtils.writeStringToFile(file, objectMapper.writeValueAsString(favorites), Charset.defaultCharset());
        } catch (IOException ex) {
            log.error("Failed to save the favorites with error" + ex.getMessage(), ex);
        }
    }

    private void loadFavorites() {
        idleTimer.playFromStart();

        // check if the cache has already been loaded
        // if so, do nothing
        synchronized (cacheLock) {
            if (cache != null)
                return;
        }

        var file = getFile();

        if (file.exists()) {
            try {
                log.debug("Loading favorites from {}", file.getAbsolutePath());

                synchronized (cacheLock) {
                    cache = objectMapper.readValue(file, Favorites.class);
                    cacheHash = cache.hashCode();
                }
            } catch (IOException ex) {
                log.error("Unable to read favorites file at " + file.getAbsolutePath(), ex);
            }
        } else {
            // build a new cache as now favorite database file is found
            synchronized (cacheLock) {
                cache = Favorites.builder().build();
                cacheHash = cache.hashCode();
            }
        }
    }

    private File getFile() {
        return new File(PopcornTimeApplication.APP_DIR + NAME);
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
