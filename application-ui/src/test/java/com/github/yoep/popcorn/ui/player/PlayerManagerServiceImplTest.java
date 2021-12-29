package com.github.yoep.popcorn.ui.player;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerAlreadyExistsException;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayerManagerServiceImplTest {
    @InjectMocks
    private PlayerManagerServiceImpl playerService;

    @Test
    void testGetById_whenIdIsNull_shouldThrowIllegalArgumentException() {
        assertThrows(IllegalArgumentException.class, () -> playerService.getById(null), "id cannot be null");
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
        var expectedResult = Optional.of(player);
        when(player.getId()).thenReturn(id);

        playerService.register(player);
        var result = playerService.getById(id);

        assertEquals(expectedResult, result);
    }

    @Test
    void testGetPlayers_whenInvoked_shouldReturnTheListOfAvailablePlayers() {
        var id1 = "player-1";
        var id2 = "player-2";
        var player1 = mock(Player.class);
        var player2 = mock(Player.class);
        var expectedResult = asList(player1, player2);
        when(player1.getId()).thenReturn(id1);
        when(player2.getId()).thenReturn(id2);

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
