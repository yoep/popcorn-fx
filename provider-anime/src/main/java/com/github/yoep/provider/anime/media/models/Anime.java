package com.github.yoep.provider.anime.media.models;

import com.github.yoep.popcorn.backend.media.providers.models.AbstractMedia;
import com.github.yoep.popcorn.backend.media.providers.models.Episode;
import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.popcorn.backend.media.providers.models.MediaType;
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
                 Images images, String synopsis) {
        super(nyaaId, imdbId, title, year, runtime, genres, null, images, synopsis);
        Objects.requireNonNull(nyaaId, "nyaaId cannot be null");
        this.nyaaId = nyaaId;
        this.episodes = episodes;
    }

    @Override
    public MediaType getType() {
        return MediaType.SHOW;
    }
}