package com.github.yoep.torrent.frostwire.model;

import com.frostwire.jlibtorrent.AlertListener;
import com.frostwire.jlibtorrent.TorrentHandle;
import com.frostwire.jlibtorrent.alerts.Alert;
import com.frostwire.jlibtorrent.alerts.AlertType;
import com.frostwire.jlibtorrent.alerts.ScrapeFailedAlert;
import com.frostwire.jlibtorrent.alerts.ScrapeReplyAlert;
import lombok.Getter;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.util.function.Consumer;

@Slf4j
@ToString
public class FrostTorrentHealth implements AlertListener {
    private final TorrentHandle handle;
    private final Consumer<FrostTorrentHealth> onComplete;

    @Getter
    private int seeds;
    @Getter
    private int peers;

    //region Constructors

    public FrostTorrentHealth(TorrentHandle handle, Consumer<FrostTorrentHealth> onComplete) {
        Assert.notNull(handle, "handle cannot be null");
        Assert.notNull(onComplete, "onComplete cannot be null");
        this.handle = handle;
        this.onComplete = onComplete;

        init();
    }

    //endregion

    //region AlertListener

    @Override
    public int[] types() {
        return new int[]{
                AlertType.TRACKER_ANNOUNCE.swig(),
                AlertType.SCRAPE_REPLY.swig(),
                AlertType.SCRAPE_FAILED.swig()
        };
    }

    @Override
    public void alert(Alert<?> alert) {
        try {
            switch (alert.type()) {
                case TRACKER_ANNOUNCE:
                    handle.scrapeTracker();
                    break;
                case SCRAPE_REPLY:
                    onScrapeRetrieved((ScrapeReplyAlert) alert);
                    break;
                case SCRAPE_FAILED:
                    onScrapeFailed((ScrapeFailedAlert) alert);
                    break;
                default:
                    //no-op
                    break;
            }
        } catch (Exception ex) {
            log.error("An error occurred while processing an alert, " + ex.getMessage(), ex);
        }
    }

    //endregion

    //region Functions

    private void init() {
        handle.resume();
    }

    private void onScrapeRetrieved(ScrapeReplyAlert alert) {
        seeds = alert.getComplete();
        peers = alert.getIncomplete();

        handle.pause();
        onComplete.accept(this);
    }

    private void onScrapeFailed(ScrapeFailedAlert alert) {
    }

    //endregion
}
