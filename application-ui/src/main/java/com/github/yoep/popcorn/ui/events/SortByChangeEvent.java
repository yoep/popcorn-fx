package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.ui.view.models.SortBy;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import org.springframework.context.ApplicationEvent;
import org.springframework.util.Assert;

@Getter
@EqualsAndHashCode(callSuper = false)
public class SortByChangeEvent extends ApplicationEvent {
    /**
     * The sort by that has been selected.
     */
    private final SortBy sortBy;

    public SortByChangeEvent(Object source, SortBy sortBy) {
        super(source);
        Assert.notNull(sortBy, "sortBy cannot be null");
        this.sortBy = sortBy;
    }
}
