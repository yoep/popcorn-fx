package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PreDestroy;
import java.util.*;
import java.util.concurrent.*;
import java.util.stream.Collectors;

/**
 * Implementation of the {@link PlayerManagerService} which serves the individual players with a central point of management.
 * This service manages each available {@link Player} of the application.
 */
@Slf4j
public class PlayerManagerServiceImpl extends AbstractListenerService<PlayerManagerListener> implements PlayerManagerService {
    private final FxChannel fxChannel;
    private final EventPublisher eventPublisher;

    private final Queue<FXPlayer> playerWrappers = new ConcurrentLinkedQueue<>();

    public PlayerManagerServiceImpl(FxChannel fxChannel, EventPublisher eventPublisher) {
        Objects.requireNonNull(fxChannel, "fxChannel cannot be null");
        Objects.requireNonNull(eventPublisher, "eventPublisher cannot be null");
        this.fxChannel = fxChannel;
        this.eventPublisher = eventPublisher;
        init();
    }

    //region Properties

    @Override
    public Optional<Player> getById(String id) {
        Objects.requireNonNull(id, "id cannot be null");
        try {
            return Optional.ofNullable(fxChannel.send(GetPlayerByIdRequest.newBuilder()
                                    .setId(id)
                                    .build(), GetPlayerByIdResponse.parser())
                            .get(5, TimeUnit.SECONDS)
                            .getPlayer())
                    .map(PlayerWrapper::new);
        } catch (ExecutionException | InterruptedException | TimeoutException e) {
            throw new RuntimeException(e);
        }
    }

    @Override
    public CompletableFuture<Collection<Player>> getPlayers() {
        return fxChannel.send(GetPlayersRequest.getDefaultInstance(), GetPlayersResponse.parser())
                .thenApply(response ->
                        response.getPlayersList().stream()
                                .map(PlayerWrapper::new)
                                .collect(Collectors.toList())
                );
    }

    @Override
    public CompletableFuture<Optional<Player>> getActivePlayer() {
        return fxChannel.send(GetActivePlayerRequest.getDefaultInstance(), GetActivePlayerResponse.parser())
                .thenApply(response -> Optional.ofNullable(response.getPlayer())
                        .map(PlayerWrapper::new));
    }

    @Override
    public void setActivePlayer(Player activePlayer) {
        Objects.requireNonNull(activePlayer, "activePlayer is required");
        log.trace("Activating player {} for playbacks", activePlayer);
        if (activePlayer instanceof PlayerWrapper(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player proto)) {
            fxChannel.send(UpdateActivePlayerRequest.newBuilder()
                    .setPlayer(proto)
                    .build());
        }
    }

    //endregion

    //region Methods

    @Override
    public void register(Player player) {
        Objects.requireNonNull(player, "player cannot be null");
        log.trace("Registering new player {}", player);
        PlayerWrapper wrapper;

        if (player instanceof PlayerWrapper e) {
            wrapper = e;
        } else {
            wrapper = PlayerWrapper.from(player);
        }

        fxChannel.send(RegisterPlayerRequest.newBuilder()
                        .setPlayer(wrapper.proto())
                        .build(), RegisterPlayerResponse.parser())
                .whenComplete((response, throwable) -> {
                    if (throwable == null) {
                        if (response.getResult() == Response.Result.OK) {
                            playerWrappers.add(new FXPlayer(player, wrapper));
                        } else {
                            log.error("Failed to register a new player, {}", response.getError());
                        }
                    } else {
                        log.error("Failed to register player", throwable);
                    }
                });
    }

    @Override
    public void unregister(Player player) {
        log.trace("Removing player \"{}\"", player);
        playerWrappers.stream()
                .filter(e -> Objects.equals(e.wrapper().getId(), player.getId()))
                .findFirst()
                .ifPresent(this::removePlayer);
    }

    private void removePlayer(FXPlayer player) {
        playerWrappers.remove(player);
        fxChannel.send(RemovePlayerRequest.newBuilder()
                .setPlayer(player.wrapper().proto())
                .build());
    }

//    @Override
//    public void callback(PlayerManagerEvent.ByValue event) {
//        log.debug("Received player manager event {}", event);
//        try (event) {
//            invokeListeners(listener -> {
//                switch (event.getTag()) {
//                    case ACTIVE_PLAYER_CHANGED -> {
//                        var change = event.getUnion().getPlayerChanged_body().playerChangedEvent;
//                        listener.activePlayerChanged(new PlayerChanged(change.getOldPlayerId().orElse(null), change.getNewPlayerId(),
//                                change.getNewPlayerName()));
//                    }
//                    case PLAYERS_CHANGED -> listener.playersChanged();
//                    case PLAYER_PLAYBACK_CHANGED -> listener.onPlayerPlaybackChanged(event.getUnion().getPlayerPlaybackChanged_body().getRequest());
//                    case PLAYER_TIME_CHANGED -> listener.onPlayerTimeChanged(event.getUnion().getPlayerTimeChanged_body().getTime());
//                    case PLAYER_DURATION_CHANGED -> listener.onPlayerDurationChanged(event.getUnion().getPlayerDurationChanged_body().getDuration());
//                    case PLAYER_STATE_CHANGED -> listener.onPlayerStateChanged(event.getUnion().getPlayerStateChanged_body().getState());
//                }
//            });
//        }
//    }

    //endregion

    //region OnDestroy

    @PreDestroy
    void onDestroy() {
        log.debug("Disposing all player resources");

    }

    //endregion

    void init() {
        registerCallbackHandler();
        registerEventListeners();
    }

    private void registerCallbackHandler() {

    }

    private void registerEventListeners() {
        eventPublisher.register(ClosePlayerEvent.class, closePlayerEvent -> {
            if (closePlayerEvent.getReason() == ClosePlayerEvent.Reason.USER) {
                getActivePlayer().whenComplete((player, throwable) -> {
                    if (throwable == null) {
                        player.ifPresent(Player::stop);
                    } else {
                        log.error("Failed to retrieve active player, {}", throwable.getMessage(), throwable);
                    }
                });
            }

            return closePlayerEvent;
        }, EventPublisher.HIGHEST_ORDER);
    }

    private record FXPlayer(Player player, PlayerWrapper wrapper) {
    }
}
