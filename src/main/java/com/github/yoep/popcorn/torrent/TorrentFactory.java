package com.github.yoep.popcorn.torrent;

import com.frostwire.jlibtorrent.AlertListener;
import com.frostwire.jlibtorrent.SessionManager;
import com.frostwire.jlibtorrent.TorrentHandle;
import com.frostwire.jlibtorrent.alerts.AddTorrentAlert;
import com.frostwire.jlibtorrent.alerts.Alert;
import com.frostwire.jlibtorrent.alerts.AlertType;
import com.github.yoep.popcorn.torrent.listeners.TorrentCreationListener;
import com.github.yoep.popcorn.torrent.listeners.TorrentListenerHolder;
import org.springframework.util.Assert;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

import static java.util.Arrays.asList;

public class TorrentFactory implements AlertListener {
    private final List<TorrentCreationListener> listeners = new ArrayList<>();
    private final SessionManager torrentSession;
    private final TorrentListenerHolder torrentListenerHolder;
    private Torrent currentTorrent;

    /**
     * Initialize a new instance of the torrent factory for creating torrents.
     *
     * @param torrentSession The torrent session manager that will be used by this factory to create new torrents.
     * @param listeners      The listeners of this factory.
     */
    TorrentFactory(SessionManager torrentSession, TorrentListenerHolder torrentListenerHolder, TorrentCreationListener... listeners) {
        Assert.notNull(torrentSession, "torrentSession cannot be null");
        this.torrentSession = torrentSession;
        this.torrentListenerHolder = torrentListenerHolder;
        this.listeners.addAll(asList(listeners));

        this.torrentSession.addListener(this);
    }

    Optional<Torrent> getCurrentTorrent() {
        return Optional.ofNullable(currentTorrent);
    }

    @Override
    public int[] types() {
        return new int[]{
                AlertType.ADD_TORRENT.swig()
        };
    }

    @Override
    public void alert(Alert<?> alert) {
        AddTorrentAlert addTorrentAlert = (AddTorrentAlert) alert;
        TorrentHandle torrentHandle = this.torrentSession.find(addTorrentAlert.handle().infoHash());
        getCurrentTorrent().ifPresent(this.torrentSession::removeListener);
        currentTorrent = new Torrent(torrentHandle, torrentListenerHolder, 15728640L);
        this.torrentSession.addListener(currentTorrent);
        torrentHandle.resume();

        listeners.forEach(e -> e.onTorrentCreated(currentTorrent));
    }
}
