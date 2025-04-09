package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.media.favorites.FavoriteEventCallback;
import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.media.watched.WatchedEventCallback;

import java.util.concurrent.CompletableFuture;

public interface OverlayItemMetadataProvider {
    boolean isLiked(Media media);

    void addListener(FavoriteEventCallback callback);

    void removeListener(FavoriteEventCallback callback);

    CompletableFuture<Boolean> isWatched(Media media);

    void addListener(WatchedEventCallback callback);

    void removeListener(WatchedEventCallback callback);
}
