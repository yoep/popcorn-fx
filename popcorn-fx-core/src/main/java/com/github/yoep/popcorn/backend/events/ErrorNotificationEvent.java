package com.github.yoep.popcorn.backend.events;

public class ErrorNotificationEvent extends NotificationEvent {
    public ErrorNotificationEvent(Object source, String text) {
        super(source, text);
    }
}
