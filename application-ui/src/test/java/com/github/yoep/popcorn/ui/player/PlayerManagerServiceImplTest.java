package com.github.yoep.popcorn.ui.player;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerAlreadyExistsException;
import com.github.yoep.popcorn.backend.player.PlayerManagerServiceImpl;
import com.github.yoep.popcorn.backend.player.PlayerSet;
import com.github.yoep.popcorn.backend.player.PlayerWrapper;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;
import java.util.stream.Stream;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayerManagerServiceImplTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @InjectMocks
    private PlayerManagerServiceImpl playerService;

    @Test
    void testGetById_whenIdIsNull_shouldThrowIllegalArgumentException() {
        assertThrows(NullPointerException.class, () -> playerService.getById(null), "id cannot be null");
    }

    @Test
    void testGetById_whenIdDoesNotExist_shouldReturnEmpty() {
        var id = "my-random-id";
        var expectedResult = Optional.empty();

        var result = playerService.getById(id);

        assertEquals(expectedResult, result);
    }

    @Test
    void testGetById_whenIdExists_shouldReturnThePlayerOfTheId() {
        var id = "player-1";
        var player = mock(Player.class);
        var wrapper = new PlayerWrapper(player);
        var expectedResult = Optional.of(wrapper);
        when(player.getId()).thenReturn(id);
        when(fxLib.player_by_id(isA(PopcornFx.class), isA(String.class))).thenReturn(wrapper);

        playerService.register(player);
        var result = playerService.getById(id);

        assertEquals(expectedResult, result);
        verify(fxLib).player_by_id(instance, id);
    }

    @Test
    void testGetPlayers_whenInvoked_shouldReturnTheListOfAvailablePlayers() {
        var id1 = "player-1";
        var id2 = "player-2";
        var player1 = mock(PlayerWrapper.class);
        var player2 = mock(PlayerWrapper.class);
        var playerSet = mock(PlayerSet.class);
        var expectedResult = Stream.of(player1, player2)
                .map(e -> (Player) e)
                .toList();
        when(player1.getId()).thenReturn(id1);
        when(player2.getId()).thenReturn(id2);
        when(playerSet.getPlayers()).thenReturn(expectedResult);
        when(fxLib.players(isA(PopcornFx.class))).thenReturn(playerSet);

        playerService.register(player1);
        playerService.register(player2);
        var result = playerService.getPlayers();

        assertTrue(result.containsAll(expectedResult), "Expected the players to be present");
    }

    @Test
    void testGetActivePlayer_whenAPlayerIsActive_shouldReturnTheActivePlayer() {
        var player = mock(Player.class);

        playerService.setActivePlayer(player);
        var result = playerService.getActivePlayer();

        assertTrue(result.isPresent(), "Expected a player to be active");
        assertEquals(player, result.get());
    }

    @Test
    void testGetActivePlayer_whenNoPlayerIsActive_shouldReturnEmpty() {
        playerService.setActivePlayer(null);
        var result = playerService.getActivePlayer();

        assertTrue(result.isEmpty(), "Expected no player to be active");
    }

    @Test
    void testRegister_whenPlayerIsNull_shouldThrowIllegalArgumentException() {
        assertThrows(IllegalArgumentException.class, () -> playerService.register(null), "player cannot be null");
    }

    @Test
    void testRegister_whenPlayerIsAlreadyExists_shouldThrowPlayerAlreadyExistsException() {
        var id = "my-unique-id";
        var registeredPlayer = mock(Player.class);
        var player = mock(Player.class);
        when(registeredPlayer.getId()).thenReturn(id);
        when(player.getId()).thenReturn(id);

        playerService.register(registeredPlayer);
        assertThrows(PlayerAlreadyExistsException.class, () -> playerService.register(player));
    }

    @Test
    void testUnregister_whenPlayerExists_shouldRemoveThePlayerFromTheList() {
        var id = "lorem";
        var player = mock(Player.class);
        when(player.getId()).thenReturn(id);

        playerService.register(player);
        assertTrue(playerService.getById(id).isPresent(), "Expected player to be registered");

        playerService.unregister(player);
        var result = playerService.getById(id);

        assertFalse(result.isPresent(), "Expected player to have been removed from the available list");
    }

    @Test
    void testOnDestroy_whenInvoked_shouldCallDisposeOnEachPlayer() {
        var id = "dolor";
        var player = mock(Player.class);
        when(player.getId()).thenReturn(id);

        playerService.register(player);
        playerService.onDestroy();

        verify(player).dispose();
    }
}
