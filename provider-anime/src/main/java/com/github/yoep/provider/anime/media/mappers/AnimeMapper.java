package com.github.yoep.provider.anime.media.mappers;

import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.Show;
import com.github.yoep.provider.anime.media.models.Anime;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;

import java.util.Objects;

@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class AnimeMapper {
    public static Movie toMovie(Anime anime) {
        Objects.requireNonNull(anime, "anime cannot be null");
        return Movie.builder()
                .id(anime.getId())
                .year(anime.getYear())
                .title(anime.getTitle())
                .images(Images.builder().build())
                .build();
    }

    public static Show toShow(Anime anime) {
        Objects.requireNonNull(anime, "anime cannot be null");
        return Show.builder()
                .id(anime.getId())
                .year(anime.getYear())
                .title(anime.getTitle())
                .status("Unknown")
                .runtime(0)
                .images(Images.builder().build())
                .numberOfSeasons(1)
                .episodes(anime.getEpisodes())
                .build();
    }
}