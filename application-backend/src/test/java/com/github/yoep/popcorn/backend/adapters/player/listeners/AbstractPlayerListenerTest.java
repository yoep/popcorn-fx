package com.github.yoep.popcorn.backend.adapters.player.listeners;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import org.junit.jupiter.api.Test;

class AbstractPlayerListenerTest {
    @Test
    void testListener() {
        var listener = new AbstractPlayerListener() {
        };

        listener.onDurationChanged(1L);
        listener.onTimeChanged(2L);
        listener.onStateChanged(Player.State.BUFFERING);
        listener.onVolumeChanged(80);
    }
}