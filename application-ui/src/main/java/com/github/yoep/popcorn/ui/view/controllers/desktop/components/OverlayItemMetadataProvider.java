package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteEventListener;
import com.github.yoep.popcorn.backend.media.watched.WatchedEventListener;

import java.util.concurrent.CompletableFuture;

public interface OverlayItemMetadataProvider {
    CompletableFuture<Boolean> isLiked(Media media);

    void addFavoriteListener(FavoriteEventListener listener);

    void removeFavoriteListener(FavoriteEventListener listener);

    CompletableFuture<Boolean> isWatched(Media media);

    void addWatchedListener(WatchedEventListener listener);

    void removeWatchedListener(WatchedEventListener listener);
}
