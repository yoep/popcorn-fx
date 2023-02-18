package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import org.springframework.util.Assert;

@Getter
@EqualsAndHashCode(callSuper = false)
public class ShowMovieDetailsEvent extends ShowDetailsEvent {
    /**
     * The media to show the details of.
     */
    private final MovieDetails media;

    @Builder
    public ShowMovieDetailsEvent(Object source, MovieDetails media) {
        super(source);
        Assert.notNull(media, "media cannot be null");
        this.media = media;
    }
}
