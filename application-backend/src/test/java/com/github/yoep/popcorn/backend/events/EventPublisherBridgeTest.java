package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Event;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import com.google.protobuf.Parser;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.doAnswer;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class EventPublisherBridgeTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private FxChannel fxChannel;
    private EventPublisherBridge bridge;

    private final AtomicReference<FxCallback<Event>> subscriptionHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocations -> {
            subscriptionHolder.set((FxCallback<Event>) invocations.getArgument(2, FxCallback.class));
            return null;
        }).when(fxChannel).subscribe(eq(FxChannel.typeFrom(Event.class)), isA(Parser.class), isA(FxCallback.class));

        bridge = new EventPublisherBridge(eventPublisher, fxChannel);
    }

    @Test
    void testOnEvent() {
        var eventListener = new AtomicReference<PlayerStartedEvent>();
        eventPublisher.register(PlayerStartedEvent.class, event -> {
            eventListener.set(event);
            return event;
        });

        subscriptionHolder.get().callback(Event.newBuilder()
                .setType(Event.EventType.PLAYER_STARTED)
                .build());

        var result = eventListener.get();
        assertNotNull(result, "expected the event listener to have been invoked");
    }

    @Test
    void testOnApplicationEvent() {
        var state = Player.State.BUFFERING;
        var request = new AtomicReference<Event>();
        doAnswer(invocations -> {
            request.set(invocations.getArgument(0, Event.class));
            return null;
        }).when(fxChannel).send(isA(Event.class));

        eventPublisher.publishEvent(new PlayerStateEvent(this, state));

        verify(fxChannel).send(isA(Event.class));
        assertEquals(Event.EventType.PLAYBACK_STATE_CHANGED, request.get().getType());
        assertEquals(state, request.get().getPlaybackStateChanged().getNewState());
    }
}