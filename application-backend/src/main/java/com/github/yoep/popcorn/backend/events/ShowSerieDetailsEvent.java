package com.github.yoep.popcorn.backend.events;

import com.github.yoep.popcorn.backend.media.providers.models.ShowDetails;
import lombok.Builder;
import lombok.Getter;

@Getter
public class ShowSerieDetailsEvent extends ShowDetailsEvent<ShowDetails> {
    @Builder
    public ShowSerieDetailsEvent(Object source, ShowDetails media) {
        super(source, media);
    }
}
