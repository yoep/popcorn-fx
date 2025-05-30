package com.github.yoep.popcorn.backend.adapters.player;

import com.github.yoep.popcorn.backend.player.PlayerManagerListener;
import com.github.yoep.popcorn.backend.services.ListenerService;

import java.util.Collection;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

/**
 * The player manager service is responsible for managing the available players which can be used by the application.
 */
public interface PlayerManagerService extends ListenerService<PlayerManagerListener> {

    /**
     * Get a list of available players.
     *
     * @return Returns a list of the current available players.
     */
    CompletableFuture<Collection<Player>> getPlayers();

    /**
     * Get the current active player which is being used for playback.
     *
     * @return Returns the active playback player, or else {@link Optional#empty()}.
     */
    CompletableFuture<Optional<Player>> getActivePlayer();

    /**
     * Set the player which should be used for video playback.
     *
     * @param activePlayer The player to use for the playbacks.
     */
    void setActivePlayer(Player activePlayer);

    /**
     * Register a new player with a unique ID.
     *
     * @param player The player that needs to be registered.
     */
    CompletableFuture<Boolean> register(Player player);

    /**
     * Remove the the player from the available players list.
     *
     * @param player The player that needs to be removed.
     */
    void unregister(Player player);
}
