package com.github.yoep.provider.anime.media.models;

import com.github.yoep.popcorn.backend.media.providers.models.AbstractMedia;
import com.github.yoep.popcorn.backend.media.providers.models.Images;
import com.github.yoep.popcorn.backend.media.providers.models.MediaType;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

import java.util.List;

@Getter
@ToString(callSuper = true)
@EqualsAndHashCode(callSuper = true)
public class Anime extends AbstractMedia {
    @Builder
    public Anime(String id, String imdbId, String title, String year,
                 Integer runtime, List<String> genres,
                 Images images, String synopsis) {
        super(id, imdbId, title, year, runtime, genres, null, images, synopsis);
    }

    @Override
    public MediaType getType() {
        return MediaType.SHOW;
    }
}
