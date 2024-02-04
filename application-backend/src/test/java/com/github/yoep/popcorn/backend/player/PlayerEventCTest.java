package com.github.yoep.popcorn.backend.player;

import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertEquals;

@ExtendWith(MockitoExtension.class)
class PlayerEventCTest {
    @Test
    void testDuration() {
        var duration = 2000L;
        var event = PlayerEventC.ByValue.durationChanged(duration);

        var result = event.getUnion().getDurationChanged_body().getDuration();

        assertEquals(duration, result);
    }

    @Test
    void testTime() {
        var time = 254000L;
        var event = PlayerEventC.ByValue.timeChanged(time);

        var result = event.getUnion().getTimeChanged_body().getTime();

        assertEquals(time, result);
    }

    @Test
    void testState() {
        var state = PlayerState.PLAYING;
        var event = PlayerEventC.ByValue.stateChanged(state);

        var result = event.getUnion().getStateChanged_body().getState();

        assertEquals(state, result);
    }
}