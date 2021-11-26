package com.github.yoep.player.popcorn.services;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.player.adapter.Player;
import com.github.yoep.player.adapter.PlayerManagerService;
import com.github.yoep.player.popcorn.PopcornPlayer;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class RegisterServiceTest {
    @Mock
    private PlayerManagerService playerService;
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private Pane embeddablePlayer;

    private RegisterService service;

    @BeforeEach
    void setUp() {
        when(viewLoader.load(RegisterService.PLAYER_SECTION_VIEW)).thenReturn(embeddablePlayer);
    }

    @Test
    void testConstructor_whenNewInstanceIsInitialized_shouldLoadThePopcornPlayerSectionView() {
        verify(viewLoader).load(RegisterService.PLAYER_SECTION_VIEW);
    }

    @Test
    void testGetPlayer_whenInvoked_shouldReturnTheCreatedPlayer() {
        var expectedPlayer = new PopcornPlayer(embeddablePlayer);
        when(viewLoader.load(RegisterService.PLAYER_SECTION_VIEW)).thenReturn(embeddablePlayer);
        service = new RegisterService(playerService, viewLoader);

        var result = service.getPlayer();

        assertNotNull(result, "Expected a popcorn player to have been created");
        assertEquals(expectedPlayer, result);
    }

    @Test
    void testInit_whenInvoked_shouldRegisterTheCreatedPopcornPlayer() {
        service = new RegisterService(playerService, viewLoader);

        service.init();

        verify(playerService).register(isA(PopcornPlayer.class));
    }

    @Test
    void testInit_whenInvoked_shouldActiveThePlayerByDefault() {
        var playerHolder = new AtomicReference<Player>();
        service = new RegisterService(playerService, viewLoader);
        doAnswer(invocation -> {
            playerHolder.set(invocation.getArgument(0, Player.class));
            return null;
        }).when(playerService).register(isA(Player.class));

        service.init();

        verify(playerService).setActivePlayer(playerHolder.get());
    }
}
