package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.ui.view.models.Category;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import org.springframework.context.ApplicationEvent;

/**
 * Invoked when the UI category is being changed.
 */
@EqualsAndHashCode(callSuper = false)
public class CategoryChangedEvent extends ApplicationEvent {
    @Getter
    private final Category category;

    public CategoryChangedEvent(Object source, Category category) {
        super(source);
        this.category = category;
    }
}
