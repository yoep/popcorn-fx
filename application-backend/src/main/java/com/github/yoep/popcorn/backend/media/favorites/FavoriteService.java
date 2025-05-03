package com.github.yoep.popcorn.backend.media.favorites;

import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.media.MediaHelper;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
import java.util.concurrent.CompletableFuture;

@Slf4j
public class FavoriteService extends AbstractListenerService<FavoriteEventListener> {
    private final FxChannel fxChannel;

    public FavoriteService(FxChannel fxChannel) {
        Objects.requireNonNull(fxChannel, "fxChannel cannot be null");
        this.fxChannel = fxChannel;
        init();
    }

    /**
     * Check if the given {@link Media} is liked by the user.
     *
     * @param favorable The favorable to check.
     * @return Returns true if the favorable is liked, else false.
     */
    public CompletableFuture<Boolean> isLiked(Media favorable) {
        Objects.requireNonNull(favorable, "favorable cannot be null");
        return fxChannel.send(GetIsLikedRequest.newBuilder()
                        .setItem(MediaHelper.getItem(favorable))
                        .build(), GetIsLikedResponse.parser())
                .thenApply(GetIsLikedResponse::getIsLiked);
    }

    /**
     * Add the given {@link Media} to the favorites.
     *
     * @param favorable The favorable to add.
     */
    public void addToFavorites(Media favorable) {
        Objects.requireNonNull(favorable, "favorable cannot be null");
        fxChannel.send(AddFavoriteRequest.newBuilder()
                        .setItem(MediaHelper.getItem(favorable))
                        .build(), AddFavoriteResponse.parser())
                .whenComplete((response, throwable) -> {
                    if (throwable == null) {
                        if (response.getResult() == Response.Result.ERROR) {
                            log.warn("Failed to add media item to favorites, {}", response.getError().getType());
                        }
                    } else {
                        log.error("Failed to add favorite", throwable);
                    }
                });
    }

    /**
     * Remove the given favorable from favorites.
     *
     * @param favorable The favorable to remove.
     */
    public void removeFromFavorites(Media favorable) {
        Objects.requireNonNull(favorable, "favorable cannot be null");
        fxChannel.send(RemoveFavoriteRequest.newBuilder()
                .setItem(MediaHelper.getItem(favorable))
                .build());
    }

    private void init() {
        fxChannel.subscribe(FxChannel.typeFrom(FavoriteEvent.class), FavoriteEvent.parser(), this::onFavoriteEvent);
    }

    private void onFavoriteEvent(FavoriteEvent event) {
        switch (event.getEvent()) {
            case LIKED_STATE_CHANGED -> invokeListeners(listener -> listener.onLikedStateChanged(event.getLikeStateChanged()));
            case UNRECOGNIZED -> log.warn("Received unrecognized favorite event {}", event);
        }
    }
}
