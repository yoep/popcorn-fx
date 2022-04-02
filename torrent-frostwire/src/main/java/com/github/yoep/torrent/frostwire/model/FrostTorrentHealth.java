package com.github.yoep.torrent.frostwire.model;

import com.frostwire.jlibtorrent.AlertListener;
import com.frostwire.jlibtorrent.TorrentHandle;
import com.frostwire.jlibtorrent.alerts.Alert;
import com.frostwire.jlibtorrent.alerts.AlertType;
import com.frostwire.jlibtorrent.alerts.ScrapeFailedAlert;
import com.frostwire.jlibtorrent.alerts.ScrapeReplyAlert;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentException;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

import java.text.MessageFormat;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;

@Slf4j
@ToString(exclude = "handle")
@EqualsAndHashCode(exclude = "handle")
public class FrostTorrentHealth implements AlertListener {
    @Getter
    private final TorrentHandle handle;
    private final CompletableFuture<FrostTorrentHealth> completableFuture = new CompletableFuture<>();

    @Getter
    private int seeds;
    @Getter
    private int peers;

    //region Constructors

    private FrostTorrentHealth(TorrentHandle handle) {
        Objects.requireNonNull(handle, "handle cannot be null");
        this.handle = handle;

        init();
    }

    //endregion

    //region Methods

    /**
     * Create a new health instance for the given handle.
     * The instance will start scraping the handle information to retrieve the health information.
     * The result of the health scraping from the handle can be retrieved through {@link FrostTorrentHealth#healthFuture()}.
     *
     * @return Returns the health instance.
     */
    public static FrostTorrentHealth create(TorrentHandle handle) {
        return new FrostTorrentHealth(handle);
    }

    /**
     * Retrieve the {@link CompletableFuture} for this health check.
     *
     * @return Returns the future of the health instance.
     */
    public CompletableFuture<FrostTorrentHealth> healthFuture() {
        return completableFuture;
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
                case TRACKER_ANNOUNCE -> handle.scrapeTracker();
                case SCRAPE_REPLY -> onScrapeRetrieved((ScrapeReplyAlert) alert);
                case SCRAPE_FAILED -> onScrapeFailed((ScrapeFailedAlert) alert);
            }
        } catch (Exception ex) {
            log.error("An error occurred while processing an alert, " + ex.getMessage(), ex);
        }
    }

    //endregion

    //region Functions

    private void init() {
        log.debug("Retrieving torrent health for \"{}\"", handle.name());
        handle.resume();
    }

    private void onScrapeRetrieved(ScrapeReplyAlert alert) {
        seeds = alert.getComplete();
        peers = alert.getIncomplete();

        onComplete();
    }

    private void onScrapeFailed(ScrapeFailedAlert alert) {
        var code = alert.error().value();
        var message = alert.errorMessage();

        log.warn("Failed to retrieve the health information, {}:{}", code, message);
        completableFuture.completeExceptionally(new TorrentException(MessageFormat.format("Failed to retrieve health information, {0}:{1}",
                code, message)));
    }

    private void onComplete() {
        var name = handle.name();

        log.debug("Health has been retrieved for \"{}\", {}", name, this);
        completableFuture.complete(this);
    }

    //endregion
}
