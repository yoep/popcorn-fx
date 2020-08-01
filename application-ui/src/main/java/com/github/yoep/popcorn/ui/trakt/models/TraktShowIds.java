package com.github.yoep.popcorn.ui.trakt.models;

import lombok.Builder;
import lombok.Data;
import lombok.EqualsAndHashCode;
import lombok.NoArgsConstructor;

@EqualsAndHashCode(callSuper = true)
@Data
@NoArgsConstructor
public class TraktShowIds extends AbstractTraktIds {
    private int tvdb;

    @Builder
    public TraktShowIds(int trakt, String slug, String imdb, int tmdb, int tvdb) {
        super(trakt, slug, imdb, tmdb);
        this.tvdb = tvdb;
    }
}
