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

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.fail;
import static org.mockito.Mockito.mock;

@ExtendWith(MockitoExtension.class)
class EventPublisherTest {
    @InjectMocks
    private EventPublisher publisher;

    @Test
    void testPublishEvent() throws ExecutionException, InterruptedException, TimeoutException {
        var event = mock(PlayMediaEvent.class);
        var mediaInvocation = new CompletableFuture<Void>();
        var torrentInvocation = new CompletableFuture<Void>();
        var stoppedInvocation = new AtomicBoolean();
        publisher.register(PlayMediaEvent.class, e -> {
            mediaInvocation.complete(null);
            return e;
        });
        publisher.register(PlayTorrentEvent.class, e -> {
            torrentInvocation.complete(null);
            return e;
        });
        publisher.register(PlayerStoppedEvent.class, e -> {
            stoppedInvocation.set(true);
            return e;
        }, EventPublisher.HIGHEST_ORDER);

        publisher.publish(event);

        mediaInvocation.get(200, TimeUnit.MILLISECONDS);
        torrentInvocation.get(200, TimeUnit.MILLISECONDS);
        assertFalse(stoppedInvocation.get());
    }

    @Test
    void testPublishEvent_whenListenerReturnsNull_shouldStopTheEventChain() throws ExecutionException, InterruptedException, TimeoutException {
        var event = mock(PlayMediaEvent.class);
        var trigger = new CompletableFuture<Void>();
        var result = new AtomicBoolean();
        publisher.register(PlayMediaEvent.class, e -> {
            trigger.complete(null);
            return null;
        }, EventPublisher.HIGHEST_ORDER);
        publisher.register(PlayMediaEvent.class, e -> {
            result.set(true);
            fail("This listener should not have been invoked");
            return e;
        }, EventPublisher.LOWEST_ORDER);

        publisher.publish(event);

        trigger.get(200, TimeUnit.MILLISECONDS);
        assertFalse(result.get());
    }
}