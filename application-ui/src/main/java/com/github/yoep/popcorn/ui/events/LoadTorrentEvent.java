package com.github.yoep.popcorn.ui.events;

/**
 * Activity which is invoked when a torrent is being loaded.
 */
public class LoadTorrentEvent extends LoadEvent {
    public LoadTorrentEvent(Object source) {
        super(source);
    }
}
