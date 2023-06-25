package com.github.yoep.popcorn.backend.events;

public class InfoNotificationEvent extends NotificationEvent {
    public InfoNotificationEvent(Object source, String text) {
        super(source, text);
    }

    public InfoNotificationEvent(Object source, String text, Runnable action) {
        super(source, text, action);
    }
}
