package com.github.yoep.torrent.frostwire.listeners;

import com.frostwire.jlibtorrent.AlertListener;
import com.frostwire.jlibtorrent.ErrorCode;
import com.frostwire.jlibtorrent.TorrentHandle;
import com.frostwire.jlibtorrent.alerts.AddTorrentAlert;
import com.frostwire.jlibtorrent.alerts.Alert;
import com.frostwire.jlibtorrent.alerts.AlertType;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
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
        Objects.requireNonNull(name, "name cannot be null");
        Objects.requireNonNull(onCompleteConsumer, "onCompleteConsumer cannot be null");
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
    public void alert(Alert<?> alert) {
        var addTorrentAlert = (AddTorrentAlert) alert;
        var torrentName = addTorrentAlert.torrentName();

        // check if the alert contains an error
        if (addTorrentAlert.error().isError()) {
            handleError(addTorrentAlert.error());
        }

        // check if this alert matches the expected torrent
        // if not, ignore this creation alert
        if (this.name.equals(torrentName)) {
            onCompleteConsumer.accept(addTorrentAlert.handle());
        }
    }

    //endregion

    //region Functions

    private void handleError(ErrorCode error) {
        log.error("An error occurred while adding a torrent (code {}), {}", error.value(), error.message());
    }

    //endregion
}
