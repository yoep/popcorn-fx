package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.doAnswer;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class EventPublisherBridgeTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @InjectMocks
    private EventPublisherBridge bridge;

    @Test
    void testOnPlayerStopped() {
        var url = "http://localhost/video.mp4";
        var quality = "720p";
        var time = 2500;
        var duration = 5000;
        var event = new PlayerStoppedEvent(this, url, null, quality, time, duration);
        var holder = new AtomicReference<EventC.ByValue>();
        doAnswer(invocation -> {
            holder.set(invocation.getArgument(1));
            return null;
        }).when(fxLib).publish_event(isA(PopcornFx.class), isA(EventC.ByValue.class));

        eventPublisher.publish(event);

        verify(fxLib).publish_event(eq(instance), isA(EventC.ByValue.class));
        var result = holder.get();
        assertEquals(EventC.Tag.PlayerStopped, result.tag);
        assertEquals(url, result.union.playerStopped_body.stoppedEvent.url);
        assertEquals(time, result.union.playerStopped_body.stoppedEvent.time.getValue());
        assertEquals(duration, result.union.playerStopped_body.stoppedEvent.duration.getValue());
    }
}