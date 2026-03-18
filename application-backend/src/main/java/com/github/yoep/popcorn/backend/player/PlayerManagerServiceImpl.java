package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.google.protobuf.ByteString;
import lombok.extern.slf4j.Slf4j;

import java.io.IOException;
import java.util.Collection;
import java.util.Objects;
import java.util.Optional;
import java.util.Queue;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.stream.Collectors;

/**
 * Implementation of the {@link PlayerManagerService} which serves the individual
 * players with a central point of management.
 * This service manages each available {@link Player} of the application.
 */
@Slf4j
public class PlayerManagerServiceImpl extends AbstractListenerService<PlayerManagerListener> implements PlayerManagerService {

    private final FxChannel fxChannel;
    private final EventPublisher eventPublisher;
    final Queue<Player> playerWrappers = new ConcurrentLinkedQueue<>();

    public PlayerManagerServiceImpl(FxChannel fxChannel, EventPublisher eventPublisher) {
        this.fxChannel = Objects.requireNonNull(fxChannel, "fxChannel cannot be null");
        this.eventPublisher = Objects.requireNonNull(eventPublisher, "eventPublisher cannot be null");
        init();
    }

    // region Properties

    @Override
    public CompletableFuture<Collection<Player>> getPlayers() {
        return fxChannel.send(GetPlayersRequest.getDefaultInstance(), GetPlayersResponse.parser())
                .thenApply(response -> response.getPlayersList().stream()
                        .map(this::toProtoWrapper)
                        .collect(Collectors.toList()));
    }

    @Override
    public CompletableFuture<Optional<Player>> getActivePlayer() {
        return fxChannel.send(GetActivePlayerRequest.getDefaultInstance(), GetActivePlayerResponse.parser())
                .thenApply(response -> Optional.of(response.getPlayer())
                        .filter(e -> response.hasPlayer())
                        .map(activePlayer -> playerWrappers.stream()
                                .filter(e -> Objects.equals(e.getId(), activePlayer.getId()))
                                .findFirst()
                                .orElseGet(() -> toProtoWrapper(activePlayer))));
    }

    @Override
    public void setActivePlayer(Player activePlayer) {
        Objects.requireNonNull(activePlayer, "activePlayer is required");
        log.trace("Activating player {} for playbacks", activePlayer);

        var request = UpdateActivePlayerRequest.newBuilder()
                .setPlayer(toProto(activePlayer))
                .build();
        fxChannel.send(request);
    }

    // endregion

    // region Methods

    @Override
    public CompletableFuture<Boolean> register(Player player) {
        Objects.requireNonNull(player, "player cannot be null");
        log.trace("Registering new player {}", player);

        if (player instanceof PlayerProtoWrapper) {
            log.error("PlayerProtoWrapper are not allowed to be registered as new players");
            return CompletableFuture.completedFuture(false);
        }

        var proto = toProto(player);
        var request = RegisterPlayerRequest.newBuilder().setPlayer(proto).build();

        return fxChannel.send(request, RegisterPlayerResponse.parser())
                .thenApply(response -> {
                    if (response.getResult() == Response.Result.OK) {
                        var playerWrapper = new PlayerFxWrapper(player, proto);
                        playerWrappers.add(playerWrapper);
                        registerPlayerEvents(playerWrapper);
                        return true;
                    } else {
                        log.error("Failed to register new player, {}", response.getError());
                        return false;
                    }
                });
    }

    @Override
    public void unregister(Player player) {
        log.trace("Removing player \"{}\"", player);
        playerWrappers.stream()
                .filter(e -> Objects.equals(e.getId(), player.getId()))
                .findFirst()
                .map(this::protoFromWrapper)
                .ifPresent(this::removePlayer);
    }

    private void removePlayer(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player player) {
        playerWrappers.removeIf(e -> Objects.equals(e.getId(), player.getId()));
        fxChannel.send(RemovePlayerRequest.newBuilder().setPlayer(player).build());
    }

    // endregion

    void onDestroy() {
        log.debug("Disposing all player resources");
        playerWrappers.forEach(Player::dispose);
    }

    void init() {
        registerCallbackHandler();
        registerEventListeners();
    }

    private void registerCallbackHandler() {
        fxChannel.subscribe(FxChannel.typeFrom(PlayerManagerEvent.class), PlayerManagerEvent.parser(), this::onPlayerManagerEvent);
        fxChannel.subscribe_response(FxChannel.typeFrom(GetPlayerStateRequest.class), GetPlayerStateRequest.parser(), this::onPlayerStateRequest);
        fxChannel.subscribe(FxChannel.typeFrom(PlayerPlayRequest.class), PlayerPlayRequest.parser(), this::onPlayerPlayRequest);
        fxChannel.subscribe(FxChannel.typeFrom(PlayerPauseRequest.class), PlayerPauseRequest.parser(), this::onPlayerPauseRequest);
        fxChannel.subscribe(FxChannel.typeFrom(PlayerResumeRequest.class), PlayerResumeRequest.parser(), this::onPlayerResumeRequest);
        fxChannel.subscribe(FxChannel.typeFrom(PlayerSeekRequest.class), PlayerSeekRequest.parser(), this::onPlayerSeekRequest);
        fxChannel.subscribe(FxChannel.typeFrom(PlayerStopRequest.class), PlayerStopRequest.parser(), this::onPlayerStopRequest);
    }

    private void registerPlayerEvents(PlayerFxWrapper player) {
        player.addListener(new PlayerListener() {
            @Override
            public void onDurationChanged(long newDuration) {
                fxChannel.send(InvokePlayerEvent.newBuilder()
                        .setId(player.getId())
                        .setEvent(PlayerEvent.newBuilder()
                                .setEvent(PlayerEvent.Event.DURATION_CHANGED)
                                .setDurationChanged(PlayerEvent.DurationChanged.newBuilder()
                                        .setDuration(newDuration)
                                        .build())
                                .build())
                        .build());
            }

            @Override
            public void onTimeChanged(long newTime) {
                fxChannel.send(InvokePlayerEvent.newBuilder()
                        .setId(player.getId())
                        .setEvent(PlayerEvent.newBuilder()
                                .setEvent(PlayerEvent.Event.TIME_CHANGED)
                                .setTimeChanged(PlayerEvent.TimeChanged.newBuilder()
                                        .setTime(newTime)
                                        .build())
                                .build())
                        .build());
            }

            @Override
            public void onStateChanged(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player.State newState) {
                fxChannel.send(InvokePlayerEvent.newBuilder()
                        .setId(player.getId())
                        .setEvent(PlayerEvent.newBuilder()
                                .setEvent(PlayerEvent.Event.STATE_CHANGED)
                                .setStateChanged(PlayerEvent.StateChanged.newBuilder()
                                        .setState(newState)
                                        .build()))
                        .build());
            }

            @Override
            public void onVolumeChanged(int volume) {
                fxChannel.send(InvokePlayerEvent.newBuilder()
                        .setId(player.getId())
                        .setEvent(PlayerEvent.newBuilder()
                                .setEvent(PlayerEvent.Event.VOLUME_CHANGED)
                                .setVolumeChanged(PlayerEvent.VolumeChanged.newBuilder()
                                        .setVolume(volume)
                                        .build())
                                .build())
                        .build());
            }
        });
    }

    private void onPlayerManagerEvent(PlayerManagerEvent message) {
        switch (message.getEvent()) {
            case ACTIVE_PLAYER_CHANGED ->
                    invokeListeners(listener -> listener.activePlayerChanged(message.getActivePlayerChanged()));
            case PLAYERS_CHANGED -> invokeListeners(PlayerManagerListener::playersChanged);
            case PLAYER_PLAYBACK_CHANGED ->
                    invokeListeners(l -> l.onPlayerPlaybackChanged(message.getPlayerPlaybackChanged().getRequest()));
            case PLAYER_DURATION_CHANGED ->
                    invokeListeners(l -> l.onPlayerDurationChanged(message.getPlayerDurationChanged().getDuration()));
            case PLAYER_TIMED_CHANGED ->
                    invokeListeners(l -> l.onPlayerTimeChanged(message.getPlayerTimeChanged().getTime()));
            case PLAYER_STATE_CHANGED ->
                    invokeListeners(l -> l.onPlayerStateChanged(message.getPlayerStateChanged().getState()));
            case UNRECOGNIZED ->
                    log.error("Failed to process player manager event, invalid event {}", message.getEvent());
        }
    }

    private void onPlayerStateRequest(Integer sequenceId, GetPlayerStateRequest request) {
        playerWrappers.stream()
                .filter(e -> Objects.equals(e.getId(), request.getPlayerId()))
                .findFirst()
                .ifPresent(e -> {
                    var state = e.getState();
                    fxChannel.send(GetPlayerStateResponse.newBuilder().setState(state).build(), sequenceId);
                });
    }

    private void onPlayerPlayRequest(PlayerPlayRequest request) {
        playerWrappers.stream()
                .filter(e -> Objects.equals(e.getId(), request.getPlayerId()))
                .findFirst()
                .ifPresent(e -> e.play(request.getRequest()));
    }

    private void onPlayerPauseRequest(PlayerPauseRequest request) {
        playerWrappers.stream()
                .filter(e -> Objects.equals(e.getId(), request.getPlayerId()))
                .findFirst()
                .ifPresent(Player::pause);
    }

    private void onPlayerResumeRequest(PlayerResumeRequest request) {
        playerWrappers.stream()
                .filter(e -> Objects.equals(e.getId(), request.getPlayerId()))
                .findFirst()
                .ifPresent(Player::resume);
    }

    private void onPlayerSeekRequest(PlayerSeekRequest request) {
        playerWrappers.stream()
                .filter(e -> Objects.equals(e.getId(), request.getPlayerId()))
                .findFirst()
                .ifPresent(e -> e.seek(request.getTime()));
    }

    private void onPlayerStopRequest(PlayerStopRequest request) {
        playerWrappers.stream()
                .filter(e -> Objects.equals(e.getId(), request.getPlayerId()))
                .findFirst()
                .ifPresent(Player::stop);
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

    private com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player protoFromWrapper(Player wrapper) {
        if (wrapper instanceof PlayerProtoWrapper protoWrapper) {
            return protoWrapper.proto();
        } else if (wrapper instanceof PlayerFxWrapper fxWrapper) {
            return fxWrapper.proto();
        }
        log.warn("Unable to convert player to proto from {}", wrapper);
        return null;
    }

    private PlayerProtoWrapper toProtoWrapper(com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player proto) {
        return new PlayerProtoWrapper(proto, fxChannel);
    }

    private static com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player toProto(Player player) {
        return com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player.newBuilder()
                .setId(player.getId())
                .setName(player.getName())
                .setDescription(player.getDescription())
                .setGraphicResource(player.getGraphicResource().map(stream -> {
                    try {
                        return ByteString.readFrom(stream);
                    } catch (IOException e) {
                        log.error("Failed to read image stream", e);
                        return ByteString.empty();
                    }
                }).orElse(ByteString.empty()))
                .setState(player.getState())
                .build();
    }
}