package com.github.yoep.torrent.frostwire;

import com.frostwire.jlibtorrent.*;
import com.github.yoep.popcorn.backend.adapters.torrent.InvalidTorrentSessionStateException;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentException;
import com.github.yoep.popcorn.backend.adapters.torrent.state.SessionState;
import javafx.beans.property.ReadOnlyObjectProperty;
import javafx.beans.property.ReadOnlyObjectWrapper;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;

@Slf4j
@RequiredArgsConstructor
public class TorrentSessionManagerImpl implements TorrentSessionManager {
    public static final String STATE_PROPERTY = "state";

    private final ReadOnlyObjectWrapper<SessionState> state = new ReadOnlyObjectWrapper<>(this, STATE_PROPERTY, SessionState.CREATING);

    private SessionManager session;
    private TorrentException error;

    //region TorrentSessionManager

    @Override
    public SessionState getState() {
        return state.get();
    }

    @Override
    public ReadOnlyObjectProperty<SessionState> stateProperty() {
        return state.getReadOnlyProperty();
    }

    @Override
    public TorrentException getError() {
        return error;
    }

    @Override
    public SessionManager getSession() {
        checkSessionState();
        return session;
    }

    @Override
    public TorrentHandle find(Sha1Hash sha1) {
        checkSessionState();
        return session.find(sha1);
    }

    @Override
    public void addListener(AlertListener listener) {
        checkSessionState();
        session.addListener(listener);
    }

    @Override
    public void removeListener(AlertListener listener) {
        checkSessionState();
        session.removeListener(listener);
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        try {
            var startTime = System.currentTimeMillis();
            initializeSession();
            initializeDht();
            log.info("Torrent session initialized in {} seconds", (System.currentTimeMillis() - startTime) / 1000.0);
        } catch (TorrentException ex) {
            log.error(ex.getMessage(), ex);
            state.set(SessionState.ERROR);
            error = ex;
        } catch (Exception ex) {
            var message = "Failed to create torrent session, " + ex.getMessage();
            log.error(message, ex);
            state.set(SessionState.ERROR);
            error = new TorrentException(message, ex);
        }
    }

    private void initializeSession() {
        log.debug("Initializing torrent session");
        state.set(SessionState.INITIALIZING);

        try {
            var sessionParams = new SessionParams();
            session = new SessionManager();

            log.trace("Starting torrent session");
            session.start(sessionParams);
        } catch (Exception ex) {
            throw new TorrentException("Failed to initialize torrent session with error: " + ex.getMessage(), ex);
        }
    }

    private void initializeDht() {
        log.debug("Initialize torrent DHT");

        // start the DHT of the torrent session
        log.trace("Starting torrent DHT");
        session.startDht();

        try {
            // wait for the dht to have at least one node before continuing
            while (session.stats().dhtNodes() > 0) {
                Thread.sleep(500);
            }

            state.set(SessionState.RUNNING);
        } catch (InterruptedException ex) {
            state.set(SessionState.ERROR);
            throw new TorrentException("The DHT monitor has unexpectedly quit", ex);
        }
    }

    //endregion

    //region PreDestroy

    @PreDestroy
    public void onDestroy() {
        if (session != null)
            session.stop();
    }

    //endregion

    //region Functions

    private void checkSessionState() {
        var state = getState();

        if (state != SessionState.RUNNING)
            throw new InvalidTorrentSessionStateException(state);
    }

    //endregion
}
