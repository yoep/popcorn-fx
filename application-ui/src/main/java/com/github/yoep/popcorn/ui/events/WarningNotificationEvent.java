package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.events.NotificationEvent;

public class WarningNotificationEvent extends NotificationEvent {
    public WarningNotificationEvent(Object source, String text) {
        super(source, text);
    }
}
