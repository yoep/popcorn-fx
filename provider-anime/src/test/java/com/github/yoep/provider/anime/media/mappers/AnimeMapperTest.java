package com.github.yoep.provider.anime.media.mappers;

import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.Show;
import com.github.yoep.provider.anime.media.models.Anime;
import org.junit.jupiter.api.Test;

import java.util.Collections;

import static org.junit.jupiter.api.Assertions.assertEquals;

class AnimeMapperTest {
    @Test
    void testToMovie_whenAnimeIsGiven_shouldReturnExpectedMovie() {
        var id = "my-id";
        var title = "my-title";
        var year = "2022";
        var anime = Anime.builder()
                .nyaaId(id)
                .title(title)
                .year(year)
                .build();
        var expectedResult = Movie.builder()
                .id(id)
                .title(title)
                .year(year)
                .images(Images.builder().build())
                .build();

        var result = AnimeMapper.toMovie(anime);

        assertEquals(expectedResult, result);
    }

    @Test
    void testToShow_whenAnimeIsGiven_shouldReturnTheExpectedShow() {
        var id = "id007";
        var title = "lorem";
        var year = "2022";
        var episode = Episode.builder()
                .episode(2)
                .build();
        var anime = Anime.builder()
                .nyaaId(id)
                .title(title)
                .year(year)
                .episodes(Collections.singletonList(episode))
                .build();
        var expectedResult = Show.builder()
                .id(id)
                .title(title)
                .year(year)
                .status("Unknown")
                .runtime(0)
                .numberOfSeasons(1)
                .episodes(Collections.singletonList(episode))
                .episodes(anime.getEpisodes())
                .images(Images.builder().build())
                .build();

        var result = AnimeMapper.toShow(anime);

        assertEquals(expectedResult, result);
    }
}