package com.github.yoep.popcorn.backend.media;

import com.github.yoep.popcorn.backend.media.providers.MediaType;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

class SeasonTest {
    @Test
    void testTitle() {
        var expectedTitle = "Season 1";
        var season = new Season(1, expectedTitle);

        assertEquals(expectedTitle, season.title());
        assertEquals(expectedTitle, season.toString());
    }

    @Test
    void testType() {
        var season = new Season(1, "1");

        assertEquals(MediaType.SHOW, season.type());
    }

    @Test
    void testOrder() {
        var season1 = new Season(1, "1");
        var season2 = new Season(2, "2");
        var season3 = new Season(3, "3");

        assertEquals(-1, season1.compareTo(season2));
        assertEquals(0, season1.compareTo(season1));
        assertEquals(1, season3.compareTo(season1));
    }
}