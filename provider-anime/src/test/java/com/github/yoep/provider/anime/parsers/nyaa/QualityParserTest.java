package com.github.yoep.provider.anime.parsers.nyaa;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class QualityParserTest {
    @Test
    void testExtractQuality_whenTitleContainsPixel_shouldReturnQuality() {
        var title = "[Tag] My Video Title - 001~131 [720p][Multiple Subtitles]";
        var expectedResult = "720p";

        var result = QualityParser.extractQuality(title);

        assertTrue(result.isPresent(), "Expected the quality to have been found");
        assertEquals(expectedResult, result.get());
    }

    @Test
    void testExtractQuality_whenTitleContainsResolution_shouldReturnQuality() {
        var title = "my video title (DVD 1920x1080)";
        var expectedResult = "1080p";

        var result = QualityParser.extractQuality(title);

        assertTrue(result.isPresent(), "Expected the quality to have been found");
        assertEquals(expectedResult, result.get());
    }

    @Test
    void testExtractQuality_whenTitleDoesNotContainAnyQualityIndication_shouldReturnEmpty() {
        var title = "[Tag 123] My video title without quality";

        var result = QualityParser.extractQuality(title);

        assertTrue(result.isEmpty(), "Expected the quality to not have been found");
    }
}