package com.github.yoep.popcorn.ui.trakt.models;

import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.NoArgsConstructor;

@Data
@NoArgsConstructor
@AllArgsConstructor
public abstract class AbstractWatched {
    private int plays;
    private String lastWatchedAt;
    private String lastUpdatedAt;
}
