package com.github.yoep.player.adapter;

import javafx.beans.property.ObjectProperty;
import javafx.beans.property.ReadOnlyMapProperty;

import java.util.Collection;
import java.util.Optional;

/**
 * The player service manages all players.
 */
public interface PlayerService {
    /**
     * Get the player by the given ID.
     *
     * @param id The id of the player to retrieve.
     * @return Returns the player if found, else {@link Optional#empty()}.
     */
    Optional<Player> getById(String id);

    /**
     * Get a list of available players.
     *
     * @return Returns a list of the current available players.
     */
    Collection<Player> getPlayers();

    /**
     * Get the players property.
     * This property manages the available players and notifies when the list is changed.
     *
     * @return Returns the player property.
     */
    ReadOnlyMapProperty<String, Player> playersProperty();

    /**
     * Get the current active player which is being used for playback.
     *
     * @return Returns the active playback player, or else {@link Optional#empty()}.
     */
    Optional<Player> getActivePlayer();

    /**
     * Get the active player property.
     *
     * @return Returns the active player property.
     */
    ObjectProperty<Player> activePlayerProperty();

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
     * @throws PlayerAlreadyExistsException Is thrown when the player ID already exists.
     */
    void register(Player player);

    /**
     * Remove the the player from the available players list.
     *
     * @param player The player that needs to be removed.
     */
    void unregister(Player player);
}
