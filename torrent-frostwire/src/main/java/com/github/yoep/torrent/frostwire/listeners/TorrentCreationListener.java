package com.github.yoep.torrent.frostwire.listeners;

import com.frostwire.jlibtorrent.AlertListener;
import com.frostwire.jlibtorrent.SessionManager;
import com.frostwire.jlibtorrent.TorrentHandle;
import com.frostwire.jlibtorrent.alerts.AddTorrentAlert;
import com.frostwire.jlibtorrent.alerts.Alert;
import com.frostwire.jlibtorrent.alerts.AlertType;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.util.function.Consumer;

/**
 * A one-time use torrent creation listener.
 */
@Slf4j
public class TorrentCreationListener implements AlertListener {
    private final SessionManager session;
    private Consumer<TorrentHandle> onCompleteConsumer;

    public TorrentCreationListener(SessionManager session) {
        Assert.notNull(session, "session cannot be null");
        this.session = session;
    }

    @Override
    public int[] types() {
        return new int[]{
                AlertType.ADD_TORRENT.swig()
        };
    }

    @Override
    public void alert(Alert<?> alert) {
        // automatically unregister this listener from the session
        unregister();

        if (onCompleteConsumer == null) {
            log.warn("A torrent creation listener was registered without a completion action");
            return;
        }

        var addTorrentAlert = (AddTorrentAlert) alert;
        var torrentHandle = session.find(addTorrentAlert.handle().infoHash());

        onCompleteConsumer.accept(torrentHandle);
    }

    /**
     * Register this listener to the torrent session.
     *
     * @return Returns this instance.
     */
    public TorrentCreationListener register() {
        session.addListener(this);
        return this;
    }

    /**
     * Set the action to execute when a torrent is being created.
     *
     * @param consumer The consumer of the completion event.
     * @return Returns this instance.
     */
    public TorrentCreationListener onComplete(Consumer<TorrentHandle> consumer) {
        onCompleteConsumer = consumer;
        return this;
    }

    /**
     * Remove this listener from the torrent session.
     */
    private void unregister() {
        session.removeListener(this);
    }
}
