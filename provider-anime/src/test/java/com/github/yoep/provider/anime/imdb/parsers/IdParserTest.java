package com.github.yoep.provider.anime.imdb.parsers;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class IdParserTest {
    @Test
    void testExtractId_whenRawHrefIsGiven_shouldExtractTheImdbId() {
        var expectedResult = "tt2560140";
        var href = "/title/" + expectedResult + "/?ref_=adv_li_tt";

        var result = IdParser.extractId(href);

        assertTrue(result.isPresent(), "Expected the id to have been found");
        assertEquals(expectedResult, result.get());
    }

    @Test
    void testExtractId_whenNoValidHrefIsGiven_shouldReturnEmpty() {
        var result = IdParser.extractId("lorem ipsum dolor");

        assertFalse(result.isPresent(), "Expected no ID to have been found");
    }
}