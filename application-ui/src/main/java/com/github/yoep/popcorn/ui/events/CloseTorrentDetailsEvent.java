package com.github.yoep.popcorn.ui.events;

import com.github.yoep.popcorn.backend.events.ApplicationEvent;
import lombok.EqualsAndHashCode;

/**
 * Invoked when the torrent details is closed.
 */
@EqualsAndHashCode(callSuper = false)
public class CloseTorrentDetailsEvent extends ApplicationEvent {
    public CloseTorrentDetailsEvent(Object source) {
        super(source);
    }
}
