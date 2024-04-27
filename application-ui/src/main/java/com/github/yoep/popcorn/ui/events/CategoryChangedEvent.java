package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.events.ApplicationEvent;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
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
    private final Category category;

    public CategoryChangedEvent(Object source, Category category) {
        super(source);
        this.category = category;
    }
}
