package com.github.yoep.popcorn.ui.trakt.models;

import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.NoArgsConstructor;

@Data
@NoArgsConstructor
@AllArgsConstructor
public abstract class AbstractTraktIds {
    private int trakt;
    private String slug;
    private String imdb;
    private int tmdb;
}
