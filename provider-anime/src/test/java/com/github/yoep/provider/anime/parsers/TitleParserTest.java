package com.github.yoep.provider.anime.parsers;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class TitleParserTest {
    @Test
    void testCleanTitle_whenContainsSubsTag_shouldRemoveSubsTag() {
        var title = "[!Subs required] my title";
        var expectedTitle = "my title";

        var result = TitleParser.cleanTitle(title);

        assertEquals(expectedTitle, result);
    }

    @Test
    void testCleanTitle_whenTitleContainsExtension_shouldRemoveExtension() {
        var title = "my video title.mkv";
        var expectedTitle = "my video title";

        var result = TitleParser.cleanTitle(title);

        assertEquals(expectedTitle, result);
    }
}