package com.github.yoep.popcorn.backend.media.favorites;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.FavoriteEvent;

public interface FavoriteEventListener {
    void onLikedStateChanged(FavoriteEvent.LikedStateChanged event);
}
