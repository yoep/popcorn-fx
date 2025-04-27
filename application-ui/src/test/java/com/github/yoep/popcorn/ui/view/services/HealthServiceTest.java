package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.ui.events.CloseDetailsEvent;
import com.google.protobuf.Parser;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class HealthServiceTest {
    @Mock
    private FxChannel fxChannel;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @InjectMocks
    private HealthService service;

    @Test
    void testCalculateHealth() {
        var seeds = 30;
        var leechers = 10;
        var request = new AtomicReference<CalculateTorrentHealthRequest>();
        when(fxChannel.send(isA(CalculateTorrentHealthRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, CalculateTorrentHealthRequest.class));
            return CompletableFuture.completedFuture(CalculateTorrentHealthResponse.newBuilder()
                    .setHealth(Torrent.Health.newBuilder()
                            .setSeeds(seeds)
                            .setLeechers(leechers)
                            .setState(Torrent.Health.State.GOOD)
                            .build())
                    .build());
        });

        var result = service.calculateHealth(seeds, leechers).resultNow();

        verify(fxChannel).send(isA(CalculateTorrentHealthRequest.class), isA(Parser.class));
        assertEquals(seeds, request.get().getSeeds());
        assertEquals(leechers, request.get().getLeechers());

        assertEquals(seeds, result.getSeeds());
        assertEquals(leechers, result.getLeechers());
        assertEquals(Torrent.Health.State.GOOD, result.getState());
    }

    @Test
    void testGetTorrentHealth() {
        var url = "magnet:?SomeTorrentUrl";
        var request = new AtomicReference<TorrentHealthRequest>();
        when(fxChannel.send(isA(TorrentHealthRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, TorrentHealthRequest.class));
            return CompletableFuture.completedFuture(TorrentHealthResponse.newBuilder()
                    .setHealth(Torrent.Health.newBuilder()
                            .setSeeds(87)
                            .setLeechers(13)
                            .setState(Torrent.Health.State.EXCELLENT)
                            .build())
                    .build());
        });

        var result = service.getTorrentHealth(url).resultNow();

        verify(fxChannel).send(isA(TorrentHealthRequest.class), isA(Parser.class));
        assertEquals(url, request.get().getUri(), "expected the request uri to match");

        assertEquals(87, result.getSeeds());
        assertEquals(13, result.getLeechers());
    }

    @Test
    void testGetTorrentHealth_cancelPrevious() {
        when(fxChannel.send(isA(TorrentHealthRequest.class), isA(Parser.class)))
                .thenReturn(new CompletableFuture<>())
                .thenReturn(CompletableFuture.completedFuture(TorrentHealthResponse.getDefaultInstance()));

        service.getTorrentHealth("magnet:?FirstTorrentUrl");
        var future = service.healthFuture;
        service.getTorrentHealth("magnet:?SecondTorrentUrl");

        assertTrue(future.isCancelled(), "expected the first future to have been cancelled");
    }

    @Test
    void testOnCloseDetailsEvent() {
        when(fxChannel.send(isA(TorrentHealthRequest.class), isA(Parser.class)))
                .thenReturn(new CompletableFuture<>());
        service.getTorrentHealth("magnet:?FooTorrentUrl");
        var future = service.healthFuture;

        eventPublisher.publishEvent(new CloseDetailsEvent(this));

        assertTrue(future.isCancelled(), "expected the future to have been cancelled");
    }
}
