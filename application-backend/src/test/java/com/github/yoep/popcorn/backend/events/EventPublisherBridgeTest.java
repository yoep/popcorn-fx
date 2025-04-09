package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.lib.FxCallback;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Event;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.PlayerState;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Optional;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class EventPublisherBridgeTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private FxChannel fxChannel;
    private EventPublisherBridge bridge;

    private final AtomicReference<FxCallback<Event>> callbackHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        bridge = new EventPublisherBridge(eventPublisher, fxChannel);
    }

    @Test
    void testPlayerState() {
        var eventHolder = new AtomicReference<EventC.ByValue>();
        var state = PlayerState.LOADING;
        doAnswer(invocation -> {
            eventHolder.set(invocation.getArgument(1, EventC.ByValue.class));
            return null;
        }).when(fxLib).publish_event(isA(PopcornFx.class), isA(EventC.ByValue.class));

        eventPublisher.publish(new PlayerStateEvent(this, state));

        verify(fxLib).publish_event(eq(instance), isA(EventC.ByValue.class));
        var result = eventHolder.get();
        assertEquals(EventC.Tag.PLAYBACK_STATE_CHANGED, result.getTag());
        assertEquals(state, result.getUnion().getPlaybackState_body().getNewState());
    }

    @Test
    void testCallback() {
        var oldPlayerId = "oldPlayerId";
        var newPlayerId = "newPlayerID";
        var newPlayerName = "newPlayerName";
        var callback = callbackHolder.get();
        var changedEvent = mock(PlayerChangedEventC.ByValue.class);
        var event = new EventC.ByValue();
        event.tag = EventC.Tag.PLAYER_CHANGED;
        event.union = new EventC.EventCUnion.ByValue();
        event.union.playerChanged_body = new EventC.PlayerChanged_Body();
        event.union.playerChanged_body.playerChangedEvent = changedEvent;
        when(changedEvent.getOldPlayerId()).thenReturn(Optional.of(oldPlayerId));
        when(changedEvent.getNewPlayerId()).thenReturn(newPlayerId);
        when(changedEvent.getNewPlayerName()).thenReturn(newPlayerName);

        callback.callback(event);

        verify(eventPublisher).publish(new PlayerChangedEvent(bridge, oldPlayerId, newPlayerId, newPlayerName));
        verify(fxLib).dispose_event_value(event);
    }
}