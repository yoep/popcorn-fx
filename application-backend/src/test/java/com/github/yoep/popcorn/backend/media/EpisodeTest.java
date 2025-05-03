package com.github.yoep.popcorn.backend.media;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class EpisodeTest {
    @Test
    void testEquals_differentProto_sameId() {
        var id = "tt00000123";
        var episode1 = new Episode(Media.Episode.newBuilder()
                .setTvdbId(id)
                .build());
        var episode2 = new Episode(Media.Episode.newBuilder()
                .setTvdbId(id)
                .build());

        assertEquals(episode1, episode2);
    }
}