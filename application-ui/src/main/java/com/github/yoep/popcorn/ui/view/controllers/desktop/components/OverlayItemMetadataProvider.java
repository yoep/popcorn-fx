package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.media.favorites.FavoriteEventCallback;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.watched.WatchedEventCallback;

public interface OverlayItemMetadataProvider {
    boolean isLiked(Media media);

    void addListener(FavoriteEventCallback callback);

    void removeListener(FavoriteEventCallback callback);

    boolean isWatched(Media media);

    void addListener(WatchedEventCallback callback);

    void removeListener(WatchedEventCallback callback);
}
