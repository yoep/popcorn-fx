package com.github.yoep.popcorn.trakt.models;

import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.NoArgsConstructor;

@Data
@NoArgsConstructor
@AllArgsConstructor
public class TraktEpisode {
    private int number;
    private int plays;
    private String lastWatchedAt;
}
