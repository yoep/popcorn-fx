package com.github.yoep.player.chromecast.discovery;

import com.github.yoep.player.chromecast.ChromecastPlayer;
import com.github.yoep.player.chromecast.services.MetaDataService;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import su.litvak.chromecast.api.v2.ChromeCast;

import java.util.Collections;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class DiscoveryServiceTest {
    @Mock
    private PlayerManagerService playerService;
    @Mock
    private ChromeCast chromeCast;
    @Mock
    private MetaDataService contentTypeService;
    @InjectMocks
    private DiscoveryService service;

    @Test
    void testNewChromeCastDiscovered_whenChromecastDeviceIsFound_shouldRegisterANewChromecastPlayer() {
        var name = "my-chromecast-device";
        var playerHolder = new AtomicReference<ChromecastPlayer>();
        when(chromeCast.getName()).thenReturn(name);
        doAnswer(invocation -> {
            playerHolder.set(invocation.getArgument(0, ChromecastPlayer.class));
            return null;
        }).when(playerService).register(isA(ChromecastPlayer.class));

        service.newChromeCastDiscovered(chromeCast);

        verify(playerService).register(isA(ChromecastPlayer.class));
        assertEquals(name, playerHolder.get().getId());
    }

    @Test
    void testChromeCastRemoved_whenChromeCastIsNotRegistered_shouldNotUnregisterPlayer() {
        when(playerService.getPlayers()).thenReturn(Collections.emptyList());

        service.chromeCastRemoved(chromeCast);

        verify(playerService, times(0)).unregister(isA(ChromecastPlayer.class));
    }

    @Test
    void testChromeCastRemoved_whenChromeCastNameDoesNotMatchId_shouldNotUnregisterPlayer() {
        var id = "my-not-matching-id";
        var name = "my-chromecast-name";
        var player = mock(ChromecastPlayer.class);
        when(playerService.getPlayers()).thenReturn(Collections.singletonList(player));
        when(player.getId()).thenReturn(id);
        when(chromeCast.getName()).thenReturn(name);

        service.chromeCastRemoved(chromeCast);

        verify(playerService, times(0)).unregister(isA(ChromecastPlayer.class));
    }

    @Test
    void testChromeCastRemoved_whenChromeCastNameMatchesTheId_shouldUnregisterTheChromecastPlayer() {
        var name = "my-chromecast-name";
        var registeredPlayer = mock(ChromecastPlayer.class);
        when(playerService.getPlayers()).thenReturn(Collections.singletonList(registeredPlayer));
        when(registeredPlayer.getId()).thenReturn(name);
        when(chromeCast.getName()).thenReturn(name);

        service.chromeCastRemoved(chromeCast);

        verify(playerService).unregister(registeredPlayer);
    }
}
