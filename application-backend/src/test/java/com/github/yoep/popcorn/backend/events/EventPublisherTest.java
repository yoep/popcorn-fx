package com.github.yoep.popcorn.backend.events;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicBoolean;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.mock;

@ExtendWith(MockitoExtension.class)
class EventPublisherTest {
    @InjectMocks
    private EventPublisher publisher;

    @Test
    void testPublishNoThreading() throws ExecutionException, InterruptedException {
        var event = mock(PlayerStoppedEvent.class);
        var future = new CompletableFuture<Boolean>();
        var eventPublisher = new EventPublisher(false);

        eventPublisher.register(PlayerStoppedEvent.class, e -> {
            future.complete(true);
            return e;
        });
        eventPublisher.publish(event);

        assertTrue(future.get());
    }

    @Test
    void testPublishEvent() throws ExecutionException, InterruptedException, TimeoutException {
        var event = mock(ShowMovieDetailsEvent.class);
        var detailsInvocation = new CompletableFuture<Void>();
        var movieDetailsInvocation = new CompletableFuture<Void>();
        var serieDetailsInvocation = new AtomicBoolean();
        publisher.register(ShowDetailsEvent.class, e -> {
            detailsInvocation.complete(null);
            return e;
        });
        publisher.register(ShowMovieDetailsEvent.class, e -> {
            movieDetailsInvocation.complete(null);
            return e;
        });
        publisher.register(ShowSerieDetailsEvent.class, e -> {
            serieDetailsInvocation.set(true);
            return e;
        }, EventPublisher.HIGHEST_ORDER);

        publisher.publish(event);

        detailsInvocation.get(200, TimeUnit.MILLISECONDS);
        movieDetailsInvocation.get(200, TimeUnit.MILLISECONDS);
        assertFalse(serieDetailsInvocation.get());
    }

    @Test
    void testPublishEvent_whenListenerReturnsNull_shouldStopTheEventChain() throws ExecutionException, InterruptedException, TimeoutException {
        var event = mock(PlayerStoppedEvent.class);
        var trigger = new CompletableFuture<Void>();
        var result = new AtomicBoolean();
        publisher.register(PlayerStoppedEvent.class, e -> {
            trigger.complete(null);
            return null;
        }, EventPublisher.HIGHEST_ORDER);
        publisher.register(PlayerStoppedEvent.class, e -> {
            result.set(true);
            fail("This listener should not have been invoked");
            return e;
        }, EventPublisher.LOWEST_ORDER);

        publisher.publish(event);

        trigger.get(200, TimeUnit.MILLISECONDS);
        assertFalse(result.get());
    }
}