package com.github.yoep.popcorn.backend.adapters.torrent;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Torrent;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class TorrentHelperTest {
    @Test
    void testGetHealthStateKey() {
        assertEquals("health_bad", TorrentHelper.getHealthStateKey(Torrent.Health.State.BAD));
        assertEquals("health_medium", TorrentHelper.getHealthStateKey(Torrent.Health.State.MEDIUM));
        assertEquals("health_good", TorrentHelper.getHealthStateKey(Torrent.Health.State.GOOD));
        assertEquals("health_excellent", TorrentHelper.getHealthStateKey(Torrent.Health.State.EXCELLENT));
    }

    @Test
    void testGetHealthStateStyleClass() {
        assertEquals("bad", TorrentHelper.getHealthStateStyleClass(Torrent.Health.State.BAD));
        assertEquals("medium", TorrentHelper.getHealthStateStyleClass(Torrent.Health.State.MEDIUM));
        assertEquals("good", TorrentHelper.getHealthStateStyleClass(Torrent.Health.State.GOOD));
        assertEquals("excellent", TorrentHelper.getHealthStateStyleClass(Torrent.Health.State.EXCELLENT));
    }
}