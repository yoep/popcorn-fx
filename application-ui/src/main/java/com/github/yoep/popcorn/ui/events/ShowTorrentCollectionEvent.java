package com.github.yoep.popcorn.ui.events;

import org.springframework.context.ApplicationEvent;

public class ShowTorrentCollectionEvent extends ApplicationEvent {
    public ShowTorrentCollectionEvent(Object source) {
        super(source);
    }
}
