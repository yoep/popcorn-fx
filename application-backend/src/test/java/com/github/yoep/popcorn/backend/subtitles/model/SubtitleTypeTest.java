package com.github.yoep.popcorn.backend.subtitles.model;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

class SubtitleTypeTest {
    @Test
    void testTypeFromExtension_whenExtensionIsSrt_shouldReturnSrtType() {
        var extension = "srt";

        var result = SubtitleType.fromExtension(extension);

        assertEquals(SubtitleType.SRT, result);
    }

    @Test
    void testTypeFromExtension_whenExtensionIsVtt_shouldReturnVttType() {
        var extension = "vtt";

        var result = SubtitleType.fromExtension(extension);

        assertEquals(SubtitleType.VTT, result);
    }

    @Test
    void testTypeFromExtension_whenExtensionIsUnknown_shouldThrowEnumException() {
        assertThrows(EnumConstantNotPresentException.class, () -> SubtitleType.fromExtension("something totally random"));
    }
}