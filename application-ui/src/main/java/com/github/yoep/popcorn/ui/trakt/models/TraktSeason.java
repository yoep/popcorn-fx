package com.github.yoep.popcorn.ui.trakt.models;

import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.NoArgsConstructor;

import java.util.List;

@Data
@NoArgsConstructor
@AllArgsConstructor
public class TraktSeason {
    private int number;
    private List<TraktEpisode> episodes;
}
