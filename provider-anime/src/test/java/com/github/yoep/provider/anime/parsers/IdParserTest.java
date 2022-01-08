package com.github.yoep.provider.anime.parsers;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class IdParserTest {
    @Test
    void testExtractId_whenViewHrefIsGiven_shouldReturnTheExpectedId() {
        var id = "1475044";
        var rawHref = "/view/" + id;

        var result = IdParser.extractId(rawHref);

        assertEquals(id, result);
    }
}