package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.ui.media.providers.models.Movie;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import org.springframework.util.Assert;

@Getter
@EqualsAndHashCode(callSuper = false)
public class ShowMovieDetailsEvent extends ShowDetailsEvent {
    /**
     * The media to show the details of.
     */
    private final Movie media;

    public ShowMovieDetailsEvent(Object source, Movie media) {
        super(source);
        Assert.notNull(media, "media cannot be null");
        this.media = media;
    }
}
