package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import lombok.Builder;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import org.springframework.util.Assert;

@Getter
@EqualsAndHashCode(callSuper = false)
public class ShowSerieDetailsEvent extends ShowDetailsEvent {
    /**
     * The media to show the details of.
     */
    private final ShowDetails media;

    @Builder
    public ShowSerieDetailsEvent(Object source, ShowDetails media) {
        super(source);
        Assert.notNull(media, "media cannot be null");
        this.media = media;
    }
}