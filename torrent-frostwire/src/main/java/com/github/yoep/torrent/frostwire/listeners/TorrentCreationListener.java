package com.github.yoep.torrent.frostwire.listeners;

import com.frostwire.jlibtorrent.AlertListener;
import com.frostwire.jlibtorrent.TorrentHandle;
import com.frostwire.jlibtorrent.alerts.Alert;
import com.frostwire.jlibtorrent.alerts.AlertType;
import com.frostwire.jlibtorrent.alerts.TorrentAlert;
import com.frostwire.jlibtorrent.swig.add_torrent_alert;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.util.function.Consumer;

/**
 * A one-time use torrent creation listener.
 */
@Slf4j
@ToString
@EqualsAndHashCode
public class TorrentCreationListener implements AlertListener {
    private final String name;
    private final Consumer<TorrentHandle> onCompleteConsumer;

    //region Constructors

    public TorrentCreationListener(String name, Consumer<TorrentHandle> onCompleteConsumer) {
        Assert.hasText(name, "name cannot be null");
        Assert.notNull(onCompleteConsumer, "onCompleteConsumer cannot be null");
        this.name = name;
        this.onCompleteConsumer = onCompleteConsumer;
    }
    //endregion

    //region AlertListener

    @Override
    public int[] types() {
        return new int[]{
                AlertType.ADD_TORRENT.swig()
        };
    }

    @Override
    @SuppressWarnings("unchecked")
    public void alert(Alert<?> alert) {
        var addTorrentAlert = (TorrentAlert<add_torrent_alert>) alert;
        var torrentName = addTorrentAlert.torrentName();

        // check if this alert matches the expected torrent
        // if not, ignore this creation alert
        if (this.name.equals(torrentName)) {
            onCompleteConsumer.accept(addTorrentAlert.handle());
        }
    }

    //endregion
}
