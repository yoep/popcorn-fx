package com.github.yoep.popcorn.trakt.models;

import lombok.Builder;
import lombok.Data;
import lombok.EqualsAndHashCode;

@EqualsAndHashCode(callSuper = true)
@Data
public class TraktMovieIds extends AbstractTraktIds {
    public TraktMovieIds() {
    }

    @Builder
    public TraktMovieIds(int trakt, String slug, String imdb, int tmdb) {
        super(trakt, slug, imdb, tmdb);
    }
}
