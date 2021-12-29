package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.events.NotificationEvent;

public class SuccessNotificationEvent extends NotificationEvent {
    public SuccessNotificationEvent(Object source, String text) {
        super(source, text);
    }
}
