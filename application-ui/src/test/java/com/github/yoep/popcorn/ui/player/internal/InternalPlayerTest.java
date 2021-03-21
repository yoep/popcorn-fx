package com.github.yoep.popcorn.ui.player.internal;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.junit.jupiter.api.Assertions.assertEquals;

@ExtendWith(MockitoExtension.class)
class InternalPlayerTest {
    @InjectMocks
    private InternalPlayer player;

    @Test
    void testGetId_whenInvoked_shouldReturnTheExpectedId() {
        var result = player.getId();

        assertEquals(InternalPlayer.PLAYER_ID, result);
    }

    @Test
    void testGetName_whenInvoked_shouldReturnTheExpectedName() {
        var result = player.getName();

        assertEquals(InternalPlayer.PLAYER_NAME, result);
    }
}
