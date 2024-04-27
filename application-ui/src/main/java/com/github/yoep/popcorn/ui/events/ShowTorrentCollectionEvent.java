package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.events.ApplicationEvent;
import lombok.EqualsAndHashCode;

@EqualsAndHashCode(callSuper = false)
public class ShowTorrentCollectionEvent extends ApplicationEvent {
    public ShowTorrentCollectionEvent(Object source) {
        super(source);
    }
}
