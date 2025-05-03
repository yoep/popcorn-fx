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

        for (var language : Subtitle.Language.values()) {
            if (language != Subtitle.Language.UNRECOGNIZED) {
                assertNotNull(SubtitleHelper.getCode(language));
            } else {
                assertThrows(IllegalArgumentException.class, () -> SubtitleHelper.getCode(language));
            }
        }
    }

    @Test
    void testGetNativeName() {
        assertEquals("Disabled", SubtitleHelper.getNativeName(Subtitle.Language.NONE));
        assertEquals("Custom", SubtitleHelper.getNativeName(Subtitle.Language.CUSTOM));
        assertEquals("English", SubtitleHelper.getNativeName(Subtitle.Language.ENGLISH));

        for (var language : Subtitle.Language.values()) {
            if (language != Subtitle.Language.UNRECOGNIZED) {
                assertNotNull(SubtitleHelper.getNativeName(language));
            } else {
                assertThrows(IllegalArgumentException.class, () -> SubtitleHelper.getCode(language));
            }
        }
    }

    @Test
    void testGetFlagResource() {
        assertEquals("/images/flags/none.png", SubtitleHelper.getFlagResource(Subtitle.Language.NONE));
        assertEquals("/images/flags/custom.png", SubtitleHelper.getFlagResource(Subtitle.Language.CUSTOM));
        assertEquals("/images/flags/ar.png", SubtitleHelper.getFlagResource(Subtitle.Language.ARABIC));
        assertEquals("/images/flags/bg.png", SubtitleHelper.getFlagResource(Subtitle.Language.BULGARIAN));
        assertEquals("/images/flags/bs.png", SubtitleHelper.getFlagResource(Subtitle.Language.BOSNIAN));
        assertEquals("/images/flags/cs.png", SubtitleHelper.getFlagResource(Subtitle.Language.CZECH));
        assertEquals("/images/flags/da.png", SubtitleHelper.getFlagResource(Subtitle.Language.DANISH));
        assertEquals("/images/flags/de.png", SubtitleHelper.getFlagResource(Subtitle.Language.GERMAN));
        assertEquals("/images/flags/el.png", SubtitleHelper.getFlagResource(Subtitle.Language.MODERN_GREEK));
        assertEquals("/images/flags/en.png", SubtitleHelper.getFlagResource(Subtitle.Language.ENGLISH));
        assertEquals("/images/flags/es.png", SubtitleHelper.getFlagResource(Subtitle.Language.SPANISH));
        assertEquals("/images/flags/et.png", SubtitleHelper.getFlagResource(Subtitle.Language.ESTONIAN));
        assertEquals("/images/flags/eu.png", SubtitleHelper.getFlagResource(Subtitle.Language.BASQUE));
        assertEquals("/images/flags/fa.png", SubtitleHelper.getFlagResource(Subtitle.Language.PERSIAN));
        assertEquals("/images/flags/fi.png", SubtitleHelper.getFlagResource(Subtitle.Language.FINNISH));
        assertEquals("/images/flags/fr.png", SubtitleHelper.getFlagResource(Subtitle.Language.FRENCH));
    }

    @Test
    void testGetSupportedFontSize() {
        var result = SubtitleHelper.supportedFontSizes();

        assertNotNull(result, "expected support font sizes to have been returned");
        assertFalse(result.isEmpty(), "expected support font sizes to have been returned");
    }
}