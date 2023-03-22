package com.github.yoep.popcorn.backend.media.favorites;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.media.FavoritesSet;
import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import java.util.Collections;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.concurrent.ConcurrentLinkedDeque;

@Slf4j
@Service
public class FavoriteService {
    private final FxLib fxLib;
    private final PopcornFx instance;

    private final Object lock = new Object();
    private final FavoriteEventCallback callback = createCallback();
    private final ConcurrentLinkedDeque<FavoriteEventCallback> listeners = new ConcurrentLinkedDeque<>();

    public FavoriteService(FxLib fxLib, PopcornFx instance) {
        this.fxLib = fxLib;
        this.instance = instance;
        init();
    }

    /**
     * Check if the given {@link Media} is liked by the user.
     *
     * @param favorable The favorable to check.
     * @return Returns true if the favorable is liked, else false.
     */
    public boolean isLiked(Media favorable) {
        Objects.requireNonNull(favorable, "favorable cannot be null");
        synchronized (lock) {
            try (var item = MediaItem.from(favorable)) {
                return fxLib.is_media_liked(instance, item) == 1;
            }
        }
    }

    /**
     * Get all the {@link Media} items that are liked by the user.
     *
     * @return Returns the list of liked items by the user.
     */
    public List<Media> getAll() {
        synchronized (lock) {
            return Optional.ofNullable(fxLib.retrieve_all_favorites(instance))
                    .map(FavoritesSet::<Media>getAll)
                    .orElse(Collections.emptyList());
        }
    }

    /**
     * Add the given {@link Media} to the favorites.
     *
     * @param favorable The favorable to add.
     */
    public void addToFavorites(Media favorable) {
        Objects.requireNonNull(favorable, "favorable cannot be null");
        synchronized (lock) {
            log.trace("Adding favorite item {}", favorable);
            try (var mediaItem = MediaItem.from(favorable)) {
                fxLib.add_to_favorites(instance, mediaItem);
            }
        }
    }

    /**
     * Remove the given favorable from favorites.
     *
     * @param favorable The favorable to remove.
     */
    public void removeFromFavorites(Media favorable) {
        Objects.requireNonNull(favorable, "favorable cannot be null");
        synchronized (lock) {
            log.trace("Removing favorite item {}", favorable);
            fxLib.remove_from_favorites(instance, MediaItem.from(favorable));
        }
    }

    public void registerListener(FavoriteEventCallback callback) {
        Objects.requireNonNull(callback, "callback cannot be null");
        listeners.add(callback);
    }

    public void removeListener(FavoriteEventCallback callback) {
        listeners.remove(callback);
    }

    private void init() {
        synchronized (lock) {
            fxLib.register_favorites_event_callback(instance, callback);
        }
    }

    private FavoriteEventCallback createCallback() {
        return event -> {
            log.debug("Received favorite event callback {}", event);
            event.close();

            for (var listener : listeners) {
                try {
                    listener.callback(event);
                } catch (Exception ex) {
                    log.error("Failed to invoke favorite callback, {}", ex.getMessage(), ex);
                }
            }
        };
    }
}
