package com.github.yoep.provider.anime.parsers;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class TitleParserTest {
    @Test
    void testNormaliseTitle_whenContainsSubsTag_shouldRemoveSubsTag() {
        var title = "[!Subs required] my title";
        var expectedTitle = "my title";

        var result = TitleParser.normaliseTitle(title);

        assertEquals(expectedTitle, result);
    }

    @Test
    void testNormaliseTitle_whenTitleContainsExtension_shouldRemoveExtension() {
        var title = "my video title.mkv";
        var expectedTitle = "my video title";

        var result = TitleParser.normaliseTitle(title);

        assertEquals(expectedTitle, result);
    }

    @Test
    void testNormaliseTitle_whenTitleContainsYear_shouldRemoveYear() {
        var title = "[Tag001] lorem ipsum - dolor (2021)";
        var expectedTitle = "lorem ipsum - dolor";

        var result = TitleParser.normaliseTitle(title);

        assertEquals(expectedTitle, result);
    }

    @Test
    void testNormaliseTitle_whenTitleContainsUnderscores_shouldReplaceWithSpace() {
        var title = "[Tag606]_lorem_ipsum_-_dolor";
        var expectedTitle = "lorem ipsum - dolor";

        var result = TitleParser.normaliseTitle(title);

        assertEquals(expectedTitle, result);
    }

    @Test
    void testNormaliseTitle_whenTitleContainsFileSize_shouldRemoveFileSize() {
        var title = "[my-tag] Video Title - Subtitle (2020) - 01 [1080p][Multiple Subtitle].mkv (1.4 GiB)";
        var expectedTitle = "Video Title - Subtitle - 01";

        var result = TitleParser.normaliseTitle(title);

        assertEquals(expectedTitle, result);
    }
}