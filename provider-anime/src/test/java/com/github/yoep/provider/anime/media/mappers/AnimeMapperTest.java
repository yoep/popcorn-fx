package com.github.yoep.provider.anime.media.mappers;

import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.provider.anime.media.models.Anime;
import org.junit.jupiter.api.Test;

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
}