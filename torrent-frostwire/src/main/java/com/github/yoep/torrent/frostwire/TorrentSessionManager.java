package com.github.yoep.torrent.frostwire;

import com.frostwire.jlibtorrent.SessionManager;
import com.frostwire.jlibtorrent.SessionParams;
import com.github.yoep.torrent.adapter.InvalidTorrentSessionStateException;
import com.github.yoep.torrent.adapter.TorrentException;
import com.github.yoep.torrent.adapter.state.SessionState;
import javafx.beans.property.ReadOnlyObjectProperty;
import javafx.beans.property.ReadOnlyObjectWrapper;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.task.TaskExecutor;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;

@Slf4j
@RequiredArgsConstructor
public class TorrentSessionManager {
    public static final String STATE_PROPERTY = "state";

    private final TaskExecutor taskExecutor;
    private final ReadOnlyObjectWrapper<SessionState> state = new ReadOnlyObjectWrapper<>(this, STATE_PROPERTY, SessionState.CREATING);

    private SessionManager session;
    private TorrentException error;

    //region Getters

    /**
     * Get the state of the torrent session.
     *
     * @return Returns the current state if the session.
     */
    SessionState getState() {
        return state.get();
    }

    /**
     * Get the state property of the session manager.
     *
     * @return Returns the state property of this session manager.
     */
    ReadOnlyObjectProperty<SessionState> stateProperty() {
        return state.getReadOnlyProperty();
    }

    /**
     * Get the error that occurred in the torrent session.
     *
     * @return Returns the exception that opccured.
     */
    TorrentException getError() {
        return error;
    }

    /**
     * Get the torrent session.
     *
     * @return Returns the torrent session if available.
     * @throws TorrentException Is thrown when the state is not {@link SessionState#RUNNING}.
     */
    SessionManager getSession() {
        checkSessionState();
        return session;
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        taskExecutor.execute(() -> {
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
        });
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
            // wait for the dht to have at least 10 nodes before continuing
            while (session.stats().dhtNodes() < 10) {
                Thread.sleep(500);
            }

            state.set(SessionState.RUNNING);
        } catch (InterruptedException ex) {
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
