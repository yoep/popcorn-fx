package com.github.yoep.popcorn.backend.media.favorites;

import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.FavoriteEvent;
import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.media.MediaHelper;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
import java.util.concurrent.ConcurrentLinkedDeque;
import java.util.concurrent.ExecutionException;

@Slf4j
public class FavoriteService implements FxCallback<FavoriteEvent> {
    private final FxChannel fxChannel;

    private final ConcurrentLinkedDeque<FxCallback<FavoriteEvent>> listeners = new ConcurrentLinkedDeque<>();

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
    public boolean isLiked(Media favorable) {
        Objects.requireNonNull(favorable, "favorable cannot be null");
        try {
            return fxChannel
                    .send(GetIsLikedRequest.newBuilder()
                            .setItem(MediaHelper.getItem(favorable))
                            .build(), GetIsLikedResponse.parser())
                    .thenApply(GetIsLikedResponse::getIsLiked)
                    .get();
        } catch (ExecutionException | InterruptedException e) {
            throw new RuntimeException(e);
        }
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
                            log.warn("Failed to add media item to favorites, {}", response.getError());
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

    public void registerListener(FxCallback<FavoriteEvent> callback) {
        Objects.requireNonNull(callback, "callback cannot be null");
        listeners.add(callback);
    }

    public void removeListener(FxCallback<FavoriteEvent> callback) {
        listeners.remove(callback);
    }

    @Override
    public void callback(FavoriteEvent message) {
        log.debug("Received favorite event callback {}", message);
        for (var listener : listeners) {
            try {
                listener.callback(message);
            } catch (Exception ex) {
                log.error("Failed to invoke favorite callback, {}", ex.getMessage(), ex);
            }
        }
    }

    private void init() {
        fxChannel.subscribe(FxChannel.typeFrom(FavoriteEvent.class), FavoriteEvent.parser(), this);
    }
}
