package com.github.yoep.popcorn.backend.events;

import lombok.EqualsAndHashCode;
import lombok.Getter;

@Getter
@EqualsAndHashCode(callSuper = false)
public abstract class NotificationEvent extends ApplicationEvent {
    /**
     * The notification text that needs to be displayed.
     */
    private final String text;
    /**
     * The action to execute when the notification is clicked.
     */
    private final Runnable action;

    public NotificationEvent(Object source, String text) {
        super(source);
        this.text = text;
        this.action = null;
    }

    public NotificationEvent(Object source, String text, Runnable action) {
        super(source);
        this.text = text;
        this.action = action;
    }
}
