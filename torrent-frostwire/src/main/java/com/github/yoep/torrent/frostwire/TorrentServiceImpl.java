package com.github.yoep.torrent.frostwire;

import com.frostwire.jlibtorrent.Priority;
import com.frostwire.jlibtorrent.TorrentHandle;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentException;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.state.SessionState;
import com.github.yoep.popcorn.backend.lib.Handle;
import com.github.yoep.popcorn.backend.torrent.CancelTorrentCallback;
import com.github.yoep.popcorn.backend.torrent.TorrentStreamEventC;
import com.github.yoep.popcorn.backend.torrent.TorrentStreamEventCallback;
import com.github.yoep.popcorn.backend.torrent.TorrentWrapper;
import com.github.yoep.torrent.frostwire.listeners.TorrentCreationListener;
import com.github.yoep.torrent.frostwire.model.FrostTorrent;
import com.github.yoep.torrent.frostwire.wrappers.TorrentInfoWrapper;
import javafx.beans.property.ReadOnlyObjectProperty;
import lombok.extern.slf4j.Slf4j;

import java.io.File;
import java.text.MessageFormat;
import java.util.*;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
public class TorrentServiceImpl implements TorrentService {
    private final TorrentSessionManager sessionManager;
    private final FxLib fxLib;
    private final PopcornFx instance;
    private final ExecutorService executorService;

    private final List<TorrentWrapper> torrentWrappers = new ArrayList<>();
    private final Map<Handle, StreamListenerHolder> torrentStreamCallbacks = new HashMap<>();

    public TorrentServiceImpl(TorrentSessionManager sessionManager, FxLib fxLib, PopcornFx instance, ExecutorService executorService) {
        this.sessionManager = sessionManager;
        this.fxLib = fxLib;
        this.instance = instance;
        this.executorService = executorService;
    }

    //region Getters

    @Override
    public SessionState getSessionState() {
        return sessionManager.getState();
    }

    @Override
    public ReadOnlyObjectProperty<SessionState> sessionStateProperty() {
        return sessionManager.stateProperty();
    }

    @Override
    public Optional<TorrentException> getSessionError() {
        return Optional.ofNullable(sessionManager.getError());
    }

    //endregion

    //region Methods

    @Override
    public CompletableFuture<Torrent> create(TorrentFileInfo torrentFile, File torrentDirectory) {
        return create(torrentFile, torrentDirectory, false);
    }

    @Override
    public CompletableFuture<Torrent> create(TorrentFileInfo torrentFile, File torrentDirectory, boolean autoStartDownload) {
        Objects.requireNonNull(torrentFile, "torrentFile cannot be null");
        Objects.requireNonNull(torrentDirectory, "torrentDirectory cannot be null");
        return CompletableFuture.supplyAsync(() -> {
            var session = sessionManager.getSession();
            var handle = internalCreateTorrentHandle(torrentFile, torrentDirectory);
            var torrent = new FrostTorrent(handle, torrentFile.getFileIndex(), autoStartDownload);

            // register the torrent in the session
            log.trace("Adding torrent \"{}\" as a listener to the torrent session", torrentFile.getFilename());
            session.addListener(torrent);

            log.debug("Torrent has been created for \"{}\"", torrentFile.getFilename());
            return torrent;
        }, executorService);
    }

    @Override
    public void remove(Torrent torrent) {
        Objects.requireNonNull(torrent, "torrent cannot be null");
        var session = sessionManager.getSession();

        // pause the torrent download
        torrent.pause();

        // check if the torrent can be removed from the session
        if (torrent instanceof FrostTorrent frostTorrent) {
            session.removeListener(frostTorrent);
            session.remove(frostTorrent.getHandle());
            log.info("Torrent \"{}\" has been removed from the torrent session", torrent.getFilename());
        } else {
            throw new TorrentException(MessageFormat.format("Invalid torrent, torrent is not a frost torrent type ({0})", torrent.getClass().getName()));
        }
    }

    @Override
    public Handle addListener(Handle handle, TorrentStreamListener listener) {
        var callback = new TorrentStreamEventCallback() {
            @Override
            public void callback(TorrentStreamEventC.ByValue event) {
                try (event) {
                    switch (event.getTag()) {
                        case STATE_CHANGED -> listener.onStateChanged(event.getUnion().getStateChanged_body().getState());
                        case DOWNLOAD_STATUS -> listener.onDownloadStatus(event.getUnion().getDownloadStatus_body().getStatus());
                    }
                } catch (Exception ex) {
                    log.error("Failed to invoked torrent stream callback listener, {}", ex.getMessage(), ex);
                }
            }
        };
        var callbackHandle = fxLib.register_torrent_stream_event_callback(instance, handle.nativeHandle(), callback);

        Optional.ofNullable(callbackHandle)
                .map(Handle::new)
                .ifPresent(e -> torrentStreamCallbacks.put(e, new StreamListenerHolder(handle, listener, callback)));

        return new Handle(callbackHandle);
    }

    @Override
    public void removeListener(Handle callbackHandle) {
        Optional.ofNullable(torrentStreamCallbacks.get(callbackHandle))
                .ifPresent(e -> {
                    fxLib.remove_torrent_stream_event_callback(instance, e.streamHandle().nativeHandle(), callbackHandle.nativeHandle());
                    torrentStreamCallbacks.remove(callbackHandle);
                });
    }

    @Override
    public void cleanup() {
        fxLib.cleanup_torrents_directory(instance);
    }

    //endregion

    //region Functions

    private TorrentHandle internalCreateTorrentHandle(TorrentFileInfo torrentFile, File torrentDirectory) {
        log.debug("Creating new torrent for {} in {}", torrentFile.getFilename(), torrentDirectory.getAbsolutePath());
        var session = sessionManager.getSession();
        var torrentInfo = (TorrentInfoWrapper) torrentFile.getTorrentInfo();
        var torrentName = torrentInfo.getName();
        var priorities = new Priority[torrentInfo.getTotalFiles()];
        var handle = new AtomicReference<TorrentHandle>();

        // check if the torrent already exists
        var existingHandle = findTorrent(torrentFile);

        // if the handle already exists, return it
        if (existingHandle.isPresent()) {
            log.trace("Found an already existing handle for {}, returning cached torrent handle", torrentName);
            return existingHandle.get();
        }

        // by default, ignore all files
        // this should prevent the torrent from starting to download immediately
        Arrays.fill(priorities, Priority.IGNORE);

        // create a new torrent creation listener
        var creationListener = new TorrentCreationListener(torrentName, torrentHandle -> {
            synchronized (this) {
                log.debug("Received torrent handle for \"{}\"", torrentFile.getFilename());
                handle.set(torrentHandle);
                notifyAll();
            }
        });

        // register the creation listener to the session
        log.trace("Adding creation listener \"{}\" to the torrent session", creationListener);
        sessionManager.addListener(creationListener);

        // start the creation of the torrent by downloading it
        session.download(torrentInfo.getNative(), torrentDirectory, null, priorities, null);

        // pause this thread and wait for the torrent to be created
        synchronized (this) {
            try {
                log.trace("Waiting for torrent handle \"{}\" to be created", torrentName);
                wait();
            } catch (InterruptedException ex) {
                log.error("Torrent creation monitor unexpectedly quit", ex);
            }
        }

        // remove the listener from the session as the creation has been completed
        log.trace("Removing creation listener \"{}\" from the torrent session", creationListener);
        sessionManager.removeListener(creationListener);

        // lookup the actual torrent handle which can be used in the session
        // and create a new Torrent instance for it
        log.trace("Looking up torrent handle in the torrent session for \"{}\"", torrentFile.getFilename());
        var torrentHandle = sessionManager.find(handle.get().infoHash());

        log.debug("Torrent handle has been created for \"{}\"", torrentFile.getFilename());
        return torrentHandle;
    }

    private Optional<TorrentHandle> findTorrent(TorrentFileInfo torrentFile) {
        var torrentInfo = (TorrentInfoWrapper) torrentFile.getTorrentInfo();
        var torrentInfoNative = torrentInfo.getNative();

        // check if we can find the torrent in the session
        // and that the handle is still valid
        // if not, return empty()
        return Optional.ofNullable(sessionManager.find(torrentInfoNative.infoHash()))
                .filter(TorrentHandle::isValid);
    }

    private CancelTorrentCallback createCancelTorrentCallback() {
        return handle -> torrentWrappers.stream()
                .filter(e -> Objects.equals(e.getHandle(), handle))
                .findFirst()
                .map(TorrentWrapper::getTorrent)
                .ifPresent(this::remove);
    }

    //endregion

    private record StreamListenerHolder(Handle streamHandle, TorrentStreamListener listener, TorrentStreamEventCallback callback) {
    }
}
