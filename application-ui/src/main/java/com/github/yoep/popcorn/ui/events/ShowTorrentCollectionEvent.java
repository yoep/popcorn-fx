package com.github.yoep.popcorn.ui.events;

import lombok.EqualsAndHashCode;
import org.springframework.context.ApplicationEvent;

@EqualsAndHashCode(callSuper = false)
public class ShowTorrentCollectionEvent extends ApplicationEvent {
    public ShowTorrentCollectionEvent(Object source) {
        super(source);
    }
}
