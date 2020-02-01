package com.github.yoep.popcorn.trakt.models;

import lombok.AllArgsConstructor;
import lombok.Builder;
import lombok.Data;
import lombok.NoArgsConstructor;

import java.util.List;

@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class AddToWatchlistRequest {
    public List<TraktMovie> movies;
    public List<TraktShow> shows;
}
