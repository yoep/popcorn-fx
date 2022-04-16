package com.github.yoep.player.chromecast.discovery;

import com.github.yoep.player.chromecast.ChromecastPlayer;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import su.litvak.chromecast.api.v2.ChromeCast;

import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class DiscoveryServiceTest {
    @Mock
    private PlayerManagerService playerService;
    @Mock
    private ChromeCast chromeCast;
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
        service.chromeCastRemoved(chromeCast);

        verify(playerService, times(0)).unregister(isA(ChromecastPlayer.class));
    }

    @Test
    void testInit_whenInvoked_shouldCreateDiscoveryThread() {
        service.init();

        assertNotNull(service.discoveryThread, "Expected a discovery thread to have been started");
    }

    @Test
    void testOnDestroy_whenInvoked_shouldStopRunningDiscoveryThread() {
        service.init();

        service.onDestroy();

        assertTrue(service.discoveryThread.isInterrupted(), "Expected a discovery thread to have been stopped");
    }
}
