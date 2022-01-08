package com.github.yoep.provider.anime.parsers;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class EpisodeParserTest {
    @Test
    void testExtractEpisode_whenEpisodeIsPresent_shouldReturnEpisode() {
        var filename = "[Tag] My Video Title - 001 [720p].mkv";
        var episode = 1;

        var result = EpisodeParser.extractEpisode(filename);

        assertEquals(episode, result);
    }
}