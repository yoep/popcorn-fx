package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.media.providers.MovieDetails;
import lombok.Builder;
import lombok.Getter;

@Getter
public class ShowMovieDetailsEvent extends ShowDetailsEvent<MovieDetails> {
    @Builder
    public ShowMovieDetailsEvent(Object source, MovieDetails media) {
        super(source, media);
    }
}
