package com.github.yoep.provider.anime.media.models;

import com.github.yoep.popcorn.backend.media.providers.models.*;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.util.List;
import java.util.Objects;

@Getter
@ToString(callSuper = true)
@EqualsAndHashCode(callSuper = true)
public class Anime extends AbstractMedia {
    /**
     * The unique ID of the nyaa server.
     */
    private final String nyaaId;

    private final List<Episode> episodes;

    @Builder
    public Anime(String nyaaId, List<Episode> episodes, String imdbId, String title, String year,
                 Integer runtime, List<String> genres,
                 Rating rating, Images images, String synopsis) {
        super(nyaaId, imdbId, title, year, runtime, genres, toRatingReference(rating), images, synopsis);
        Objects.requireNonNull(nyaaId, "nyaaId cannot be null");
        this.nyaaId = nyaaId;
        this.episodes = episodes;
    }

    @Override
    public MediaType getType() {
        return MediaType.SHOW;
    }

    /**
     * Create a builder copy of the given {@link Anime} item.
     *
     * @param anime The item to copy.
     * @return Returns the builder instance of the copied item.
     */
    public static Anime.AnimeBuilder copy(Anime anime) {
        return Anime.builder()
                .imdbId(anime.getImdbId())
                .nyaaId(anime.getNyaaId())
                .title(anime.getTitle())
                .year(anime.getYear())
                .genres(anime.getGenres())
                .synopsis(anime.getSynopsis())
                .runtime(anime.getRuntime())
                .images(anime.getImages())
                .episodes(anime.getEpisodes());
    }
}
