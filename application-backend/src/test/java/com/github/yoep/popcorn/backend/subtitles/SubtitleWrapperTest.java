package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import org.junit.jupiter.api.Test;

import java.util.Optional;

import static org.junit.jupiter.api.Assertions.*;

class SubtitleWrapperTest {
    @Test
    void testIsNone() {
        var subtitle = new SubtitleWrapper(Subtitle.newBuilder()
                .setInfo(Subtitle.Info.newBuilder()
                        .setLanguage(Subtitle.Language.NONE)
                        .build())
                .build());

        assertTrue(subtitle.isNone());
    }

    @Test
    void testGetInfo() {
        var subtitleInfo = Subtitle.Info.newBuilder()
                .setImdbId("ImdbId")
                .setLanguage(Subtitle.Language.ENGLISH)
                .build();
        var subtitle = new SubtitleWrapper(Subtitle.newBuilder()
                .setInfo(subtitleInfo)
                .build());

        assertEquals(Optional.of(subtitleInfo), subtitle.getSubtitleInfo());
    }

    @Test
    void testEquals() {
        var subtitleInfo = Subtitle.Info.newBuilder()
                .setImdbId("tt124000")
                .setLanguage(Subtitle.Language.ENGLISH)
                .build();
        var subtitle1 = new SubtitleWrapper(Subtitle.newBuilder()
                .setInfo(subtitleInfo)
                .build());
        var subtitle2 = new SubtitleWrapper(Subtitle.newBuilder()
                .setInfo(subtitleInfo)
                .build());

        assertEquals(subtitle1, subtitle2);
    }
}