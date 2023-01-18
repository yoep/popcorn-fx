package com.github.yoep.popcorn.backend.media.favorites;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import com.github.yoep.popcorn.backend.media.providers.Favorite;
import com.github.yoep.popcorn.backend.media.providers.FavoritesSet;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import java.util.Collections;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.ConcurrentLinkedDeque;

@Slf4j
@Service
public class FavoriteService {
    private final Object lock = new Object();
    private final FavoriteEventCallback callback = createCallback();
    private final ConcurrentLinkedDeque<FavoriteEventCallback> listeners = new ConcurrentLinkedDeque<>();

    public FavoriteService() {
        init();
    }

    /**
     * Check if the given {@link com.github.yoep.popcorn.backend.media.providers.models.Media} is liked by the user.
     *
     * @param favorable The favorable to check.
     * @return Returns true if the favorable is liked, else false.
     */
    public boolean isLiked(Media favorable) {
        Assert.notNull(favorable, "favorable cannot be null");
        synchronized (lock) {
            return FxLib.INSTANCE.is_media_liked(PopcornFxInstance.INSTANCE.get(), Favorite.from(favorable));
        }
    }

    /**
     * Get all the {@link Media} items that are liked by the user.
     *
     * @return Returns the list of liked items by the user.
     */
    public List<Media> getAll() {
        synchronized (lock) {
            return Optional.ofNullable(FxLib.INSTANCE.retrieve_all_favorites(PopcornFxInstance.INSTANCE.get()))
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
        Assert.notNull(favorable, "favorable cannot be null");
        synchronized (lock) {
            FxLib.INSTANCE.add_to_favorites(PopcornFxInstance.INSTANCE.get(), Favorite.from(favorable));
        }
    }

    /**
     * Remove the given favorable from favorites.
     *
     * @param favorable The favorable to remove.
     */
    public void removeFromFavorites(Media favorable) {
        Assert.notNull(favorable, "favorable cannot be null");
        synchronized (lock) {
            FxLib.INSTANCE.remove_from_favorites(PopcornFxInstance.INSTANCE.get(), Favorite.from(favorable));
        }
    }

    public void registerListener(FavoriteEventCallback callback) {
        Assert.notNull(callback, "callback cannot be null");
        listeners.add(callback);
    }

    public void removeListener(FavoriteEventCallback callback) {
        listeners.remove(callback);
    }

    private void init() {
        synchronized (lock) {
            FxLib.INSTANCE.register_favorites_event_callback(PopcornFxInstance.INSTANCE.get(), callback);
        }
    }

    private FavoriteEventCallback createCallback() {
        return event -> {
            log.debug("Received favorite event callback {}", event);

            try {
                for (var listener : listeners) {
                    listener.callback(event);
                }
            } catch (Exception ex) {
                log.error("Failed to invoke favorite callback, {}", ex.getMessage(), ex);
            }
        };
    }
}
