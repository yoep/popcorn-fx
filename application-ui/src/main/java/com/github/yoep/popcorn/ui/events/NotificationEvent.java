package com.github.yoep.popcorn.ui.events;

import lombok.EqualsAndHashCode;
import lombok.Getter;
import org.springframework.context.ApplicationEvent;

@Getter
@EqualsAndHashCode(callSuper = false)
public abstract class NotificationEvent extends ApplicationEvent {
    /**
     * The notification text that needs to be displayed.
     */
    private final String text;

    public NotificationEvent(Object source, String text) {
        super(source);
        this.text = text;
    }
}
