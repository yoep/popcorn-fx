package com.github.yoep.popcorn.ui.events;

import lombok.EqualsAndHashCode;
import lombok.Getter;
import org.springframework.context.ApplicationEvent;
import org.springframework.util.Assert;

@Getter
@EqualsAndHashCode(callSuper = false)
public class SearchEvent extends ApplicationEvent {
    /**
     * The search value of this search activity.
     */
    private final String value;

    public SearchEvent(Object source, String value) {
        super(source);
        Assert.notNull(value, "value cannot be null");
        this.value = value;
    }
}
