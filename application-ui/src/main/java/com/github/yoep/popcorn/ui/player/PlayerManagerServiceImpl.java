package com.github.yoep.popcorn.ui.player;

import com.github.yoep.player.adapter.Player;
import com.github.yoep.player.adapter.PlayerAlreadyExistsException;
import com.github.yoep.player.adapter.PlayerManagerService;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.collections.FXCollections;
import javafx.collections.ObservableMap;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import javax.annotation.PreDestroy;
import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.Optional;

/**
 * Implementation of the {@link PlayerManagerService} which serves the individual players with a central point of management.
 * This service manages each available {@link Player} of the application.
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class PlayerManagerServiceImpl implements PlayerManagerService {
    public static final String ACTIVE_PLAYER_PROPERTY = "activePlayer";

    private final ObservableMap<String, Player> players = FXCollections.observableMap(new LinkedHashMap<>());
    private final ObjectProperty<Player> activePlayer = new SimpleObjectProperty<>(this, ACTIVE_PLAYER_PROPERTY);

    //region Properties

    @Override
    public Optional<Player> getById(String id) {
        Assert.notNull(id, "id cannot be null");
        return Optional.ofNullable(players.get(id));
    }

    @Override
    public Collection<Player> getPlayers() {
        return players.values();
    }

    @Override
    public ObservableMap<String, Player> playersProperty() {
        return players;
    }

    @Override
    public Optional<Player> getActivePlayer() {
        return Optional.ofNullable(activePlayer.get());
    }

    @Override
    public ObjectProperty<Player> activePlayerProperty() {
        return activePlayer;
    }

    @Override
    public void setActivePlayer(Player activePlayer) {
        log.trace("Activating player {} for playbacks", activePlayer);
        this.activePlayer.set(activePlayer);
    }

    //endregion

    //region Methods

    @Override
    public void register(Player player) {
        Assert.notNull(player, "player cannot be null");
        log.trace("Registering new player {}", player);
        var id = player.getId();

        // check if the player already exists with the given name
        // if so, throw an exception that the player already exists
        if (players.containsKey(id)) {
            throw new PlayerAlreadyExistsException(id);
        }

        players.put(id, player);
    }

    @Override
    public void unregister(Player player) {
        log.trace("Removing player \"{}\"", player);
        var id = player.getId();

        players.remove(id);
    }

    //endregion

    //region OnDestroy

    @PreDestroy
    void onDestroy() {
        players.values().forEach(Player::dispose);
    }

    //endregion
}
