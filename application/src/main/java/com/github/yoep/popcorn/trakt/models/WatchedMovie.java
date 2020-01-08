package com.github.yoep.popcorn.trakt.models;

import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.EqualsAndHashCode;
import lombok.NoArgsConstructor;

@EqualsAndHashCode(callSuper = true)
@Data
@NoArgsConstructor
@AllArgsConstructor
public class WatchedMovie extends AbstractWatched {
    private TraktMovie movie;
}
