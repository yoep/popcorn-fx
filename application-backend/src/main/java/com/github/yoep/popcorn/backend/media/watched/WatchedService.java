package com.github.yoep.popcorn.backend.media.watched;

import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.GetIsWatchedRequest;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.GetIsWatchedResponse;
import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.media.MediaHelper;
import com.github.yoep.popcorn.backend.media.watched.models.Watchable;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentLinkedDeque;

/**
 * The watched service maintains all the watched {@link Media} items of the application.
 * This is done through the {@link Watchable} items that are received from events and marking them as watched.
 */
@Slf4j
public class WatchedService {
    private final FxChannel fxChannel;

    private final ConcurrentLinkedDeque<WatchedEventCallback> listeners = new ConcurrentLinkedDeque<>();

    public WatchedService(FxChannel fxChannel) {
        this.fxChannel = fxChannel;
        init();
    }

    //region Methods

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
        // TODO
    }

    /**
     * Remove the watchable item from the watched list.
     *
     * @param watchable The watchable item to remove.
     */
    public void removeFromWatchList(Media watchable) {
        Objects.requireNonNull(watchable, "watchable cannot be null");
        // TODO
    }

    public void registerListener(WatchedEventCallback callback) {
        Objects.requireNonNull(callback, "callback cannot be null");
        listeners.add(callback);
    }

    public void removeListener(WatchedEventCallback callback) {
        listeners.remove(callback);
    }

    //endregion
    private void init() {
        // TODO callback
    }

    private WatchedEventCallback createCallback() {
        return event -> {
            log.debug("Received watched event callback {}", event);

            try {
                for (var listener : listeners) {
                    listener.callback(event);
                }
            } catch (Exception ex) {
                log.error("Failed to invoke watched callback, {}", ex.getMessage(), ex);
            }
        };
    }
}
