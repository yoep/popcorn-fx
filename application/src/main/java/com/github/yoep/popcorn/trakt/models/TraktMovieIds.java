package com.github.yoep.popcorn.trakt.models;

import lombok.Builder;
import lombok.Data;
import lombok.EqualsAndHashCode;
import lombok.NoArgsConstructor;

@EqualsAndHashCode(callSuper = true)
@Data
@NoArgsConstructor
public class TraktMovieIds extends AbstractTraktIds {
    @Builder
    public TraktMovieIds(int trakt, String slug, String imdb, int tmdb) {
        super(trakt, slug, imdb, tmdb);
    }
}
