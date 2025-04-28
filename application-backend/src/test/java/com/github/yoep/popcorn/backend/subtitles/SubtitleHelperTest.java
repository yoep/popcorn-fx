package com.github.yoep.popcorn.backend.subtitles;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Subtitle;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SubtitleHelperTest {
    @Test
    void testGetCode() {
        assertEquals("none", SubtitleHelper.getCode(Subtitle.Language.NONE));
        assertEquals("custom", SubtitleHelper.getCode(Subtitle.Language.CUSTOM));
        assertEquals("ar", SubtitleHelper.getCode(Subtitle.Language.ARABIC));
        assertEquals("en", SubtitleHelper.getCode(Subtitle.Language.ENGLISH));
    }

    @Test
    void testGetNativeName() {
        assertEquals("Disabled", SubtitleHelper.getNativeName(Subtitle.Language.NONE));
        assertEquals("Custom", SubtitleHelper.getNativeName(Subtitle.Language.CUSTOM));
        assertEquals("العربية", SubtitleHelper.getNativeName(Subtitle.Language.ARABIC));
        assertEquals("English", SubtitleHelper.getNativeName(Subtitle.Language.ENGLISH));
    }

    @Test
    void testGetFlagResource() {
        assertEquals("/images/flags/none.png", SubtitleHelper.getFlagResource(Subtitle.Language.NONE));
        assertEquals("/images/flags/custom.png", SubtitleHelper.getFlagResource(Subtitle.Language.CUSTOM));
        assertEquals("/images/flags/en.png", SubtitleHelper.getFlagResource(Subtitle.Language.ENGLISH));
        assertEquals("/images/flags/fr.png", SubtitleHelper.getFlagResource(Subtitle.Language.FRENCH));
        assertEquals("/images/flags/de.png", SubtitleHelper.getFlagResource(Subtitle.Language.GERMAN));
    }

    @Test
    void testGetSupportedFontSize() {
        var result = SubtitleHelper.supportedFontSizes();

        assertNotNull(result, "expected support font sizes to have been returned");
        assertFalse(result.isEmpty(), "expected support font sizes to have been returned");
    }
}