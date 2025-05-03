package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SubtitleInfoWrapperTest {
    @Test
    void testIsNone() {
        var subtitle = new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                .setLanguage(Subtitle.Language.NONE)
                .build());

        assertTrue(subtitle.isNone());
    }

    @Test
    void testIsCustom() {
        var subtitle = new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                .setLanguage(Subtitle.Language.CUSTOM)
                .build());

        assertTrue(subtitle.isCustom());
    }

    @Test
    void testGetFlagResource() {
        var subtitle = new SubtitleInfoWrapper(Subtitle.Info.newBuilder()
                .setLanguage(Subtitle.Language.ENGLISH)
                .build());

        assertEquals("/images/flags/en.png", subtitle.getFlagResource());
    }
}