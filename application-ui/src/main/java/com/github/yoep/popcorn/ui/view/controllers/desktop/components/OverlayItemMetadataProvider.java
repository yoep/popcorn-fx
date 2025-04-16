package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.FavoriteEvent;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteEventCallback;
import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.media.watched.WatchedEventCallback;

import java.util.concurrent.CompletableFuture;

public interface OverlayItemMetadataProvider {
    boolean isLiked(Media media);

    void addListener(FxCallback<FavoriteEvent> callback);

    void removeListener(FxCallback<FavoriteEvent> callback);

    CompletableFuture<Boolean> isWatched(Media media);

    void addListener(WatchedEventCallback callback);

    void removeListener(WatchedEventCallback callback);
}
