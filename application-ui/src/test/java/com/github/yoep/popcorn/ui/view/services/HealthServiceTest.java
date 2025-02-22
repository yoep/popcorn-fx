package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentHealthResult;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentHealth;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.ui.events.CloseDetailsEvent;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class HealthServiceTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @InjectMocks
    private HealthService healthService;

    @Test
    void testCalculateHealth_whenInvoked_shouldCallCalculateHealthOnTorrentService() {
        var seeds = 10;
        var peers = 20;
        var expectedResult = new TorrentHealth.ByReference();
        when(fxLib.calculate_torrent_health(isA(PopcornFx.class), isA(Integer.class), isA(Integer.class))).thenReturn(expectedResult);

        var result = healthService.calculateHealth(seeds, peers);

        verify(fxLib).calculate_torrent_health(instance, seeds, peers);
        assertEquals(expectedResult, result);
    }

    @Test
    void testGetTorrentHealth_whenPreviousFutureIsStillRunning_shouldCancelPreviousFuture() {
        var firstUrl = "lorem";
        var secondUrl = "ipsum";
        lenient().when(fxLib.calculate_torrent_health(eq(instance), anyInt(), anyInt()))
                .thenAnswer(invocation -> {
                    // how to sleep thread
                    try {
                        Thread.sleep(1000);
                    } catch (InterruptedException e) {
                        throw new RuntimeException(e);
                    }

                    return new TorrentHealth();
                });

        healthService.getTorrentHealth(firstUrl);
        var future = healthService.healthFuture;
        healthService.getTorrentHealth(secondUrl);

        assertTrue(future.isCancelled());
    }

    @Test
    void testOnLoadMediaTorrent_whenPreviousFutureIsStillRunning_shouldCancelPreviousFuture() {
        var firstUrl = "lorem";
        var wait = new CompletableFuture<Void>();
        lenient().doAnswer(invocations -> {
            Thread.sleep(1000);
            return new TorrentHealthResult.ByValue();
        }).when(fxLib).torrent_health_from_uri(isA(PopcornFx.class), isA(String.class));
        eventPublisher.register(CloseDetailsEvent.class, event -> {
            wait.complete(null);
            return null;
        }, EventPublisher.LOWEST_ORDER);

        var future = healthService.getTorrentHealth(firstUrl);
        eventPublisher.publish(new CloseDetailsEvent(this));

        assertTrue(future.isCancelled());
    }
}
