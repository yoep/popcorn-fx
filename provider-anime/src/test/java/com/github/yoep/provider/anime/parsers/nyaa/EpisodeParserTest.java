package com.github.yoep.provider.anime.parsers.nyaa;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class EpisodeParserTest {
    @Test
    void testExtractEpisode_whenEpisodeIsPresent_shouldReturnEpisode() {
        var filename = "[Tag] My Video Title - 001 [720p].mkv";
        var episode = 1;

        var result = EpisodeParser.extractEpisode(filename);

        assertTrue(result.isPresent(), "Expected the episode number to have been found");
        assertEquals(episode, result.get());
    }
}