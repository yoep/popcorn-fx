package com.github.yoep.player.vlc.mappers;

import com.github.yoep.player.vlc.model.VlcState;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class StateMapperTest {
    @Test
    void testMAp_whenVlcStateIsPlaying_shouldReturnPlaying() {
        var result = StateMapper.map(VlcState.PLAYING);

        assertEquals(PlayerState.PLAYING, result);
    }

    @Test
    void testMAp_whenVlcStateIsStopped_shouldReturnStopped() {
        var result = StateMapper.map(VlcState.STOPPED);

        assertEquals(PlayerState.STOPPED, result);
    }

    @Test
    void testMAp_whenVlcStateIsPaused_shouldReturnPaused() {
        var result = StateMapper.map(VlcState.PAUSED);

        assertEquals(PlayerState.PAUSED, result);
    }
}