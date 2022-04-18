package com.github.yoep.player.vlc.model;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class VlcStateTest {
    @Test
    void testFromValue_whenStateIsPlaying_shouldReturnPlaying() {
        var result = VlcState.fromValue("playing");

        assertEquals(VlcState.PLAYING, result);
    }

    @Test
    void testFromValue_whenStateIsPaused_shouldReturnPaused() {
        var result = VlcState.fromValue("paused");

        assertEquals(VlcState.PAUSED, result);
    }

    @Test
    void testFromValue_whenStateIsStopped_shouldReturnStopped() {
        var result = VlcState.fromValue("stopped");

        assertEquals(VlcState.STOPPED, result);
    }
}