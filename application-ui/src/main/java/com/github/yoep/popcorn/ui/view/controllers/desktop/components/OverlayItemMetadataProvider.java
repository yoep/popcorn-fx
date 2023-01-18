package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.media.favorites.FavoriteEventCallback;
import com.github.yoep.popcorn.backend.media.providers.models.Media;

public interface OverlayItemMetadataProvider {
    boolean isLiked(Media media);

    void addListener(FavoriteEventCallback callback);

    void removeListener(FavoriteEventCallback callback);
}
