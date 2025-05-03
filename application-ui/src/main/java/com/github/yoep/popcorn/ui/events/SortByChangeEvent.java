package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.events.ApplicationEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import lombok.EqualsAndHashCode;
import lombok.Getter;

import java.util.Objects;

@Getter
@EqualsAndHashCode(callSuper = false)
public class SortByChangeEvent extends ApplicationEvent {
    /**
     * The sort by that has been selected.
     */
    private final Media.SortBy sortBy;

    public SortByChangeEvent(Object source, Media.SortBy sortBy) {
        super(source);
        Objects.requireNonNull(sortBy, "sortBy cannot be null");
        this.sortBy = sortBy;
    }
}
