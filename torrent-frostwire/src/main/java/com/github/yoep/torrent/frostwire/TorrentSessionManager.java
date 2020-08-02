package com.github.yoep.torrent.frostwire;

import com.frostwire.jlibtorrent.AlertListener;
import com.frostwire.jlibtorrent.SessionManager;
import com.frostwire.jlibtorrent.Sha1Hash;
import com.frostwire.jlibtorrent.TorrentHandle;
import com.github.yoep.torrent.adapter.TorrentException;
import com.github.yoep.torrent.adapter.state.SessionState;
import javafx.beans.property.ReadOnlyObjectProperty;

public interface TorrentSessionManager {
    /**
     * Get the state of the torrent session.
     *
     * @return Returns the current state if the session.
     */
    SessionState getState();

    /**
     * Get the state property of the session manager.
     *
     * @return Returns the state property of this session manager.
     */
    ReadOnlyObjectProperty<SessionState> stateProperty();

    /**
     * Get the error that occurred in the torrent session.
     *
     * @return Returns the exception that occurred if available, else null.
     */
    TorrentException getError();

    /**
     * Get the frostwire torrent session.
     *
     * @return Returns the torrent session if available.
     * @throws TorrentException Is thrown when the state is not {@link SessionState#RUNNING}.
     */
    SessionManager getSession();

    /**
     * Find the torrent handle for the given hash.
     *
     * @param sha1 The hash of the handle.
     * @return Returns the torrent handle if found, else null.
     * @throws TorrentException Is thrown when the state is not {@link SessionState#RUNNING}.
     */
    TorrentHandle find(Sha1Hash sha1);

    /**
     * Add the given listener to this session.
     *
     * @param listener The listener to register.
     * @throws TorrentException Is thrown when the state is not {@link SessionState#RUNNING}.
     */
    void addListener(AlertListener listener);

    /**
     * Remove the given listener from this session.
     *
     * @param listener The listener to remove.
     * @throws TorrentException Is thrown when the state is not {@link SessionState#RUNNING}.
     */
    void removeListener(AlertListener listener);
}
