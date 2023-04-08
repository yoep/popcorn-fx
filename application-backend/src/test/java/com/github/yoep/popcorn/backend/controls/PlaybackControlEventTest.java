package com.github.yoep.popcorn.backend.controls;

import com.sun.jna.FromNativeContext;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.mock;

class PlaybackControlEventTest {
    @Test
    void testFromNative() {
        assertEquals(PlaybackControlEvent.TogglePlaybackState, PlaybackControlEvent.TogglePlaybackState.fromNative(0, mock(FromNativeContext.class)));
        assertEquals(PlaybackControlEvent.Forward, PlaybackControlEvent.TogglePlaybackState.fromNative(1, mock(FromNativeContext.class)));
        assertEquals(PlaybackControlEvent.Rewind, PlaybackControlEvent.TogglePlaybackState.fromNative(2, mock(FromNativeContext.class)));
    }

    @Test
    void testToNative() {
        assertEquals(0, PlaybackControlEvent.TogglePlaybackState.toNative());
        assertEquals(1, PlaybackControlEvent.Forward.toNative());
    }
}