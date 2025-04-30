package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.google.protobuf.Parser;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayerManagerServiceImplTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private FxChannel fxChannel;
    private PlayerManagerServiceImpl service;

    private final AtomicReference<FxCallback<PlayerManagerEvent>> eventListenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocations -> {
            eventListenerHolder.set((FxCallback<PlayerManagerEvent>) invocations.getArgument(2, FxCallback.class));
            return null;
        }).when(fxChannel).subscribe(eq(FxChannel.typeFrom(PlayerManagerEvent.class)), isA(Parser.class), isA(FxCallback.class));

        service = new PlayerManagerServiceImpl(fxChannel, eventPublisher);
    }

    @Test
    void testGetPlayers() {
        var player = Player.newBuilder().setId("1").build();
        when(fxChannel.send(isA(GetPlayersRequest.class), isA(Parser.class))).thenReturn(CompletableFuture.completedFuture(GetPlayersResponse.newBuilder()
                .addPlayers(player)
                .build()
        ));

        var result = service.getPlayers().resultNow();

        verify(fxChannel).send(isA(GetPlayersRequest.class), isA(Parser.class));
        assertEquals(1, result.size());
        var resultPlayer = result.iterator().next();
        assertInstanceOf(PlayerProtoWrapper.class, resultPlayer);
        assertEquals(player, ((PlayerProtoWrapper) resultPlayer).proto());
    }

    @Nested
    class GetActivePlayer {
        @Test
        void testProtoPlayer() {
            var player = Player.newBuilder().setId("active").build();
            when(fxChannel.send(isA(GetActivePlayerRequest.class), isA(Parser.class))).thenReturn(CompletableFuture.completedFuture(GetActivePlayerResponse.newBuilder()
                    .setPlayer(player)
                    .build()
            ));

            var result = service.getActivePlayer().resultNow();

            verify(fxChannel).send(isA(GetActivePlayerRequest.class), isA(Parser.class));
            assertTrue(result.isPresent(), "expected an active player");
            assertInstanceOf(PlayerProtoWrapper.class, result.get());
            assertEquals(player, ((PlayerProtoWrapper) result.get()).proto());
        }

        @Test
        void testFxPlayer() {
            var id = "FxPlayerId";
            var fxPlayer = mock(com.github.yoep.popcorn.backend.adapters.player.Player.class);
            var player = Player.newBuilder()
                    .setId(id)
                    .build();
            when(fxPlayer.getId()).thenReturn(id);
            when(fxPlayer.getName()).thenReturn("fxPlayer");
            when(fxPlayer.getDescription()).thenReturn("fxPlayerDescription");
            when(fxPlayer.getState()).thenReturn(Player.State.PLAYING);
            when(fxChannel.send(isA(GetActivePlayerRequest.class), isA(Parser.class))).thenReturn(CompletableFuture.completedFuture(GetActivePlayerResponse.newBuilder()
                    .setPlayer(player)
                    .build()
            ));
            when(fxChannel.send(isA(RegisterPlayerRequest.class), isA(Parser.class))).thenReturn(CompletableFuture.completedFuture(RegisterPlayerResponse.newBuilder()
                    .setResult(Response.Result.OK)
                    .build()
            ));

            var registrationResult = service.register(fxPlayer).resultNow();
            verify(fxChannel).send(isA(RegisterPlayerRequest.class), isA(Parser.class));
            assertEquals(true, registrationResult, "expected the player to have been registered");

            var result = service.getActivePlayer().resultNow();

            verify(fxChannel).send(isA(GetActivePlayerRequest.class), isA(Parser.class));
            assertTrue(result.isPresent(), "expected an active player");
            assertInstanceOf(PlayerFxWrapper.class, result.get());
            assertEquals(fxPlayer, ((PlayerFxWrapper) result.get()).player());
        }
    }

    @Test
    void testSetActivePlayer() {
        var protoPlayer = Player.newBuilder()
                .setId("fxPlayer")
                .build();
        var player = new PlayerProtoWrapper(protoPlayer, fxChannel);
        var request = new AtomicReference<UpdateActivePlayerRequest>();
        doAnswer(invocations -> {
            request.set(invocations.getArgument(0, UpdateActivePlayerRequest.class));
            return null;
        }).when(fxChannel).send(isA(UpdateActivePlayerRequest.class));

        service.setActivePlayer(player);

        verify(fxChannel).send(isA(UpdateActivePlayerRequest.class));
        assertNotNull(request, "expected a request to have been sent");
        assertEquals(protoPlayer, request.get().getPlayer());
    }

    @Test
    void testUnregister() {
        var id = "FxPlayerId";
        var fxPlayer = mock(com.github.yoep.popcorn.backend.adapters.player.Player.class);
        when(fxPlayer.getId()).thenReturn(id);
        when(fxPlayer.getName()).thenReturn("fxPlayer");
        when(fxPlayer.getDescription()).thenReturn("fxPlayerDescription");
        when(fxPlayer.getState()).thenReturn(Player.State.PLAYING);
        when(fxChannel.send(isA(RegisterPlayerRequest.class), isA(Parser.class))).thenReturn(CompletableFuture.completedFuture(RegisterPlayerResponse.newBuilder()
                .setResult(Response.Result.OK)
                .build()
        ));

        var registrationResult = service.register(fxPlayer).resultNow();
        verify(fxChannel).send(isA(RegisterPlayerRequest.class), isA(Parser.class));
        assertEquals(1, service.playerWrappers.size(), "expected the player to have been added to the wrappers");
        assertEquals(true, registrationResult, "expected the player to have been registered");

        service.unregister(fxPlayer);

        verify(fxChannel).send(isA(RemovePlayerRequest.class));
        assertEquals(0, service.playerWrappers.size(), "expected the player to have been removed from the wrappers");
    }

    @Test
    void testOnDestroy() {
        var player = mock(com.github.yoep.popcorn.backend.adapters.player.Player.class);
        when(player.getId()).thenReturn("player-id");
        when(player.getName()).thenReturn("fxPlayer");
        when(player.getDescription()).thenReturn("fxPlayerDescription");
        when(player.getState()).thenReturn(Player.State.UNKNOWN);
        when(fxChannel.send(isA(RegisterPlayerRequest.class), isA(Parser.class))).thenReturn(CompletableFuture.completedFuture(RegisterPlayerResponse.newBuilder()
                .setResult(Response.Result.OK)
                .build()
        ));

        service.register(player).resultNow();
        verify(fxChannel).send(isA(RegisterPlayerRequest.class), isA(Parser.class));

        service.onDestroy();

        verify(player).dispose();
    }
}