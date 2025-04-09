package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.events.ApplicationEvent;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;

/**
 * Invoked when the UI category is being changed.
 */
@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
public class CategoryChangedEvent extends ApplicationEvent {
    private final Media.Category category;

    public CategoryChangedEvent(Object source, Media.Category category) {
        super(source);
        this.category = category;
    }
}
