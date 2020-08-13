package com.github.yoep.popcorn.ui.events;

import lombok.EqualsAndHashCode;
import org.springframework.context.ApplicationEvent;

@EqualsAndHashCode(callSuper = false)
public class CloseSettingsEvent extends ApplicationEvent {
    public CloseSettingsEvent(Object source) {
        super(source);
    }
}
