package com.github.yoep.player.popcorn.player;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.CompletableFuture;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class EmbeddablePopcornPlayerTest {
    @Mock
    private PopcornPlayer popcornPlayer;
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private PlayerManagerService playerManagerService;
    private EmbeddablePopcornPlayer player;

    @BeforeEach
    void setUp() {
        when(playerManagerService.register(isA(Player.class))).thenReturn(CompletableFuture.completedFuture(true));

        player = new EmbeddablePopcornPlayer(playerManagerService, viewLoader, popcornPlayer);
    }

    @Test
    void testInit_shouldRegisterPlayer() {
        verify(playerManagerService).register(player);
    }

    @Test
    void testGetId_whenInvoked_shouldReturnTheBasePlayerId() {
        var id = "lorem";
        when(popcornPlayer.getId()).thenReturn(id);

        var result = player.getId();

        assertEquals(id, result);
    }

    @Test
    void testGetName_whenInvoked_shouldReturnTheBasePlayerName() {
        var name = "ipsum";
        when(popcornPlayer.getName()).thenReturn(name);

        var result = player.getName();

        assertEquals(name, result);
    }

    @Test
    void testIsEmbeddedPlaybackSupported_whenInvoked_shouldReturnTrue() {
        var result = player.isEmbeddedPlaybackSupported();

        assertTrue(result, "Expected the embeddable popcorn player to support embedded playback");
    }
}