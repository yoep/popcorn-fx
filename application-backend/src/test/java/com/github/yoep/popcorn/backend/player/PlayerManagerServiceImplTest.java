package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;

import static java.util.Collections.singletonList;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayerManagerServiceImplTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @InjectMocks
    private PlayerManagerServiceImpl service;

    @Test
    void testGetById() {
        var playerId = "FooBar";
        var player = mock(PlayerWrapper.class);
        when(fxLib.player_by_id(isA(PopcornFx.class), isA(String.class))).thenReturn(player);

        var result = service.getById(playerId);

        assertEquals(player, result.get());
        verify(fxLib).player_by_id(instance, playerId);
    }

    @Test
    void testGetPlayers() {
        var set = mock(PlayerSet.class);
        var players = singletonList(mock(Player.class));
        when(set.getPlayers()).thenReturn(players);
        when(fxLib.players(isA(PopcornFx.class))).thenReturn(set);

        var result = service.getPlayers();

        assertEquals(players, result);
        verify(fxLib).players(instance);
    }

    @Test
    void testGetActivePlayer() {
        var player = mock(PlayerWrapper.class);
        when(fxLib.active_player(isA(PopcornFx.class))).thenReturn(player);

        var result = service.getActivePlayer();

        assertTrue(result.isPresent(), "expected an active player to have been returned");
        assertEquals(player, result.get());
        verify(fxLib).active_player(instance);
    }

    @Test
    void testSetActivePlayer() {
        var playerId = "MyPlayerId";
        var player = mock(PlayerWrapper.class);
        when(player.getId()).thenReturn(playerId);

        service.setActivePlayer(player);

        verify(fxLib).set_active_player(instance, playerId);
    }

    @Test
    void testInit() {
        verify(fxLib).register_player_callback(instance, service);
    }

    @Test
    void testOnClosePlayerEvent_whenReasonIsUser_shouldStopPlayer() {
        var player = mock(PlayerWrapper.class);
        when(fxLib.active_player(isA(PopcornFx.class))).thenReturn(player);

        eventPublisher.publish(new ClosePlayerEvent(this, ClosePlayerEvent.Reason.USER));

        verify(player).stop();
        verify(fxLib).active_player(instance);
    }
}