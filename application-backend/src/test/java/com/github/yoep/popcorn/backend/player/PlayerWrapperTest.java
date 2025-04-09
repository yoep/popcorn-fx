package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.io.ByteArrayInputStream;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayerWrapperTest {
    @Mock
    private FxLib fxLib;

    @BeforeEach
    void setUp() {
        FxLib.INSTANCE.set(fxLib);
    }

    @Test
    void testPlayerWrapperConstructor() {
        var playerId = "MyPlayerId";
        var playerName = "MyPlayerName";
        var playerDescription = "MyPlayerDescription";
        var expectedGraphicResource = new byte[]{10, 13, 99, 78};
        var graphicResource = new ByteArrayInputStream(expectedGraphicResource);
        var player = mock(Player.class);
        when(player.getId()).thenReturn(playerId);
        when(player.getName()).thenReturn(playerName);
        when(player.getDescription()).thenReturn(playerDescription);
        when(player.getGraphicResource()).thenReturn(Optional.of(graphicResource));

        var result = new PlayerWrapper(player);

        assertEquals(playerId, result.getId());
        assertEquals(playerName, result.getName());
        assertEquals(playerDescription, result.getDescription());
        assertNotNull( result.graphicResource);
        assertEquals(4, result.graphicResourceLen);
    }

    @Test
    void testClose() {
        var player = mock(Player.class);
        var ptr = mock(PlayerWrapperPointer.class);
        var wrapper = new PlayerWrapper.ByReference(player);
        wrapper.playerC = mock(PlayerWrapperPointer.class);

        wrapper.close();

        verify(fxLib, times(0)).dispose_player_pointer(ptr);
    }

    @Test
    void testDispose() {
        var player = mock(Player.class);
        var ptr = mock(PlayerWrapperPointer.class);
        var wrapper = new PlayerWrapper.ByReference(player);
        wrapper.playerC = ptr;

        wrapper.dispose();

        verify(fxLib).dispose_player_pointer(ptr);
    }
}