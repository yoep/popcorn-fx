package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import org.springframework.context.ApplicationEvent;

@Getter
@EqualsAndHashCode(callSuper = false)
public class GenreChangeEvent extends ApplicationEvent {
    /**
     * The genre that has been selected.
     */
    private final Genre genre;

    public GenreChangeEvent(Object source, Genre genre) {
        super(source);
        this.genre = genre;
    }
}
