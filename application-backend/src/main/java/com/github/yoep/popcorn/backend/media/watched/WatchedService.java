package com.github.yoep.popcorn.backend.media.watched;

import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.media.MediaHelper;
import com.github.yoep.popcorn.backend.media.watched.models.Watchable;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
import java.util.concurrent.CompletableFuture;

/**
 * The watched service maintains all the watched {@link Media} items of the application.
 * This is done through the {@link Watchable} items that are received from events and marking them as watched.
 */
@Slf4j
public class WatchedService extends AbstractListenerService<WatchedEventListener> {
    private final FxChannel fxChannel;

    public WatchedService(FxChannel fxChannel) {
        this.fxChannel = fxChannel;
        init();
    }

    /**
     * Check if the given watchable has been watched already.
     *
     * @param watchable The watchable to check the watched state for.
     * @return Returns true if the watchable has already been watched, else false.
     */
    public CompletableFuture<Boolean> isWatched(Media watchable) {
        Objects.requireNonNull(watchable, "watchable cannot be null");
        return fxChannel.send(
                        GetIsWatchedRequest.newBuilder()
                                .setItem(MediaHelper.getItem(watchable))
                                .build(),
                        GetIsWatchedResponse.parser())
                .thenApply(GetIsWatchedResponse::getIsWatched);
    }

    /**
     * Add the watchable item to the watched list.
     *
     * @param watchable the watchable item to add.
     */
    public void addToWatchList(Media watchable) {
        Objects.requireNonNull(watchable, "watchable cannot be null");
        fxChannel.send(AddToWatchlistRequest.newBuilder()
                .setItem(MediaHelper.getItem(watchable))
                .build(), AddToWatchlistResponse.parser()).whenComplete((result, throwable) -> {
            if (throwable == null) {
                if (result.getResult() == Response.Result.ERROR) {
                    log.error("Failed to add media to watchlist, {}", result.getError());
                }
            } else {
                log.error("Failed to add media to watchlist", throwable);
            }
        });
    }

    /**
     * Remove the watchable item from the watched list.
     *
     * @param watchable The watchable item to remove.
     */
    public void removeFromWatchList(Media watchable) {
        Objects.requireNonNull(watchable, "watchable cannot be null");
        fxChannel.send(RemoveFromWatchlistRequest.newBuilder()
                .setItem(MediaHelper.getItem(watchable))
                .build());
    }

    private void init() {
        fxChannel.subscribe(FxChannel.typeFrom(WatchedEvent.class), WatchedEvent.parser(), this::onWatchedEvent);
    }

    private void onWatchedEvent(WatchedEvent event) {
        switch (event.getEvent()) {
            case STATE_CHANGED -> invokeListeners(listener ->
                    listener.onWatchedStateChanged(event.getWatchedStateChanged()));
        }
    }
}
