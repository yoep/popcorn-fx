package com.github.yoep.popcorn.ui.player.internal;

import com.github.yoep.player.adapter.PlayerService;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class InternalPlayerServiceTest {
    @Mock
    private PlayerService playerService;
    @InjectMocks
    private InternalPlayerService internalPlayerService;

    @Test
    void testInit_whenInvoked_shouldRegisterTheInternalPlayer() {
        var internalPlayer = new InternalPlayer();

        internalPlayerService.init();

        verify(playerService).register(internalPlayer);
    }

    @Test
    void testInit_whenInvoked_shouldActiveTheInternalPlayerByDefault() {
        var internalPlayer = new InternalPlayer();

        internalPlayerService.init();

        verify(playerService).setActivePlayer(internalPlayer);
    }
}
