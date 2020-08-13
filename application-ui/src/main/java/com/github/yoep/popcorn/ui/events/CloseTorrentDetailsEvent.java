package com.github.yoep.popcorn.ui.events;

import lombok.EqualsAndHashCode;
import org.springframework.context.ApplicationEvent;

/**
 * Invoked when the torrent details is closed.
 */
@EqualsAndHashCode(callSuper = false)
public class CloseTorrentDetailsEvent extends ApplicationEvent {
    public CloseTorrentDetailsEvent(Object source) {
        super(source);
    }
}
