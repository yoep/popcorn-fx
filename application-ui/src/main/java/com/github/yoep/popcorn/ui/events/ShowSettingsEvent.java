package com.github.yoep.popcorn.ui.events;

import lombok.EqualsAndHashCode;
import org.springframework.context.ApplicationEvent;

@EqualsAndHashCode(callSuper = false)
public class ShowSettingsEvent extends ApplicationEvent {
    public ShowSettingsEvent(Object source) {
        super(source);
    }
}
