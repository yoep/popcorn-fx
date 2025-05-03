package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.events.ApplicationEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import lombok.EqualsAndHashCode;
import lombok.Getter;

@Getter
@EqualsAndHashCode(callSuper = false)
public class GenreChangeEvent extends ApplicationEvent {
    /**
     * The genre that has been selected.
     */
    private final Media.Genre genre;

    public GenreChangeEvent(Object source, Media.Genre genre) {
        super(source);
        this.genre = genre;
    }
}
