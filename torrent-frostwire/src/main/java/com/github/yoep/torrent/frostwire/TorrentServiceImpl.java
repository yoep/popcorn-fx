package com.github.yoep.torrent.frostwire;

import com.frostwire.jlibtorrent.Priority;
import com.frostwire.jlibtorrent.TorrentHandle;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.torrent.*;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentHealth;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.state.SessionState;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentHealthState;
import com.github.yoep.popcorn.backend.lib.Handle;
import com.github.yoep.popcorn.backend.torrent.*;
import com.github.yoep.torrent.frostwire.listeners.TorrentCreationListener;
import com.github.yoep.torrent.frostwire.model.FrostTorrent;
import com.github.yoep.torrent.frostwire.model.FrostTorrentHealth;
import com.github.yoep.torrent.frostwire.model.TorrentHealthImpl;
import com.github.yoep.torrent.frostwire.wrappers.TorrentInfoWrapper;
import javafx.beans.property.ReadOnlyObjectProperty;
import lombok.extern.slf4j.Slf4j;

import java.io.File;
import java.text.MessageFormat;
import java.util.*;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
public class TorrentServiceImpl implements TorrentService {
    private final TorrentSessionManager sessionManager;
    private final TorrentResolverService torrentResolverService;
    private final FxLib fxLib;
    private final PopcornFx instance;
    private final ExecutorService executorService;
    private final ResolveTorrentInfoCallback resolveTorrentInfoCallback = createResolveTorrentInfoCallback();
    private final ResolveTorrentCallback resolveTorrentCallback = createResolveTorrentCallback();
    private final CancelTorrentCallback cancelTorrentCallback = createCancelTorrentCallback();

    private final List<com.github.yoep.popcorn.backend.adapters.torrent.TorrentInfoWrapper> torrentInfos = new ArrayList<>();
    private final List<TorrentWrapper> torrentWrappers = new ArrayList<>();
    private final Map<Handle, StreamListenerHolder> torrentStreamCallbacks = new HashMap<>();

    public TorrentServiceImpl(TorrentSessionManager sessionManager, TorrentResolverService torrentResolverService, FxLib fxLib, PopcornFx instance, ExecutorService executorService) {
        this.sessionManager = sessionManager;
        this.torrentResolverService = torrentResolverService;
        this.fxLib = fxLib;
        this.instance = instance;
        this.executorService = executorService;
        init();
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
    public CompletableFuture<TorrentInfo> getTorrentInfo(String torrentUrl) {
        return CompletableFuture.supplyAsync(() -> torrentResolverService.resolveUrl(torrentUrl), executorService);
    }

    @Override
    public CompletableFuture<TorrentHealth> getTorrentHealth(String url, File torrentDirectory) {
        Objects.requireNonNull(url, "url cannot be empty");
        return CompletableFuture.supplyAsync(() -> {
            var torrentInfo = torrentResolverService.resolveUrl(url);
            return getTorrentHealth(torrentInfo.getLargestFile(), torrentDirectory).join();
        }, executorService);
    }

    @Override
    public CompletableFuture<TorrentHealth> getTorrentHealth(TorrentFileInfo torrentFile, File torrentDirectory) {
        Objects.requireNonNull(torrentFile, "torrentFile cannot be null");
        Objects.requireNonNull(torrentDirectory, "torrentDirectory cannot be null");
        return CompletableFuture.supplyAsync(() -> {
            var session = sessionManager.getSession();
            var handle = internalCreateTorrentHandle(torrentFile, torrentDirectory);
            var torrentHealth = FrostTorrentHealth.create(handle);

            try {
                session.addListener(torrentHealth);
                var health = torrentHealth.healthFuture().get(10, TimeUnit.SECONDS);
                return calculateHealth(health.getSeeds(), health.getPeers());
            } catch (Exception ex) {
                throw new TorrentException("Failed to get torrent health", ex);
            } finally {
                session.removeListener(torrentHealth);
            }
        }, executorService);
    }

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
    public TorrentHealth calculateHealth(int seeds, int peers) {
        // if seeds & peers are 0
        // return the state unknown
        if (seeds == 0 && peers == 0) {
            return new TorrentHealthImpl(TorrentHealthState.UNKNOWN, 0, 0, 0);
        }

        // first calculate the seed/peer ratio
        var ratio = peers > 0 ? ((float) seeds / peers) : seeds;

        // normalize the data. Convert each to a percentage
        // ratio: Anything above a ratio of 5 is good
        double normalizedRatio = Math.min(ratio / 5 * 100, 100);
        // seeds: Anything above 30 seeds is good
        double normalizedSeeds = Math.min(seeds / 30 * 100, 100);

        // weight the above metrics differently
        // ratio is weighted 60% whilst seeders is 40%
        var weightedRatio = normalizedRatio * 0.6;
        var weightedSeeds = normalizedSeeds * 0.4;
        var weightedTotal = weightedRatio + weightedSeeds;
        var scaledTotal = (int) (weightedTotal * 3 / 100);
        var healthState = switch (scaledTotal) {
            case 0 -> TorrentHealthState.BAD;
            case 1 -> TorrentHealthState.MEDIUM;
            case 2 -> TorrentHealthState.GOOD;
            case 3 -> TorrentHealthState.EXCELLENT;
            default -> TorrentHealthState.UNKNOWN;
        };

        return new TorrentHealthImpl(healthState, ratio, seeds, peers);
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

    private void init() {
        fxLib.torrent_resolve_info_callback(instance, resolveTorrentInfoCallback);
        fxLib.register_torrent_resolve_callback(instance, resolveTorrentCallback);
        fxLib.torrent_cancel_callback(instance, cancelTorrentCallback);
    }

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

    private ResolveTorrentInfoCallback createResolveTorrentInfoCallback() {
        return url -> {
            log.debug("Executing resolve torrent info callback for {}", url);
            try {
                var torrentInfo = getTorrentInfo(url).get();
                var info = new com.github.yoep.popcorn.backend.adapters.torrent.TorrentInfoWrapper.ByValue(torrentInfo);
                torrentInfos.add(info);
                return new TorrentInfoResult.ByValue(info);
            } catch (Exception ex) {
                log.error("Failed to resolve torrent info, {}", ex.getMessage(), ex);
                // do not throw any errors here as this will crash the rust lib
                var error = new TorrentError.ByValue();
                error.tag = TorrentError.Tag.TORRENT_RESOLVING_FAILED;
                error.union = new TorrentError.TorrentErrorUnion.ByValue();
                error.union.torrentResolvingFailed_body = new TorrentError.TorrentResolvingFailed_Body(ex.getMessage());
                return new TorrentInfoResult.ByValue(error);
            }
        };
    }

    private ResolveTorrentCallback createResolveTorrentCallback() {
        return (fileInfo, torrentDirectory, autoStartDownload) -> {
            log.debug("Executing resolve torrent callback for {}", fileInfo);
            var torrentFile = torrentInfos.stream()
                    .flatMap(info -> info.getFiles().stream())
                    .filter(file -> file.equals(fileInfo))
                    .findFirst()
                    .orElseThrow(() -> new TorrentException("Torrent file couldn't be found back"));

            try {
                var torrent = create(torrentFile, new File(torrentDirectory), autoStartDownload == 1).get();
                var wrapper = new TorrentWrapper.ByValue(instance, torrent);
                torrentWrappers.add(wrapper);
                return wrapper;
            } catch (Exception ex) {
                log.error("Failed to resolve torrent, {}", ex.getMessage(), ex);
                throw new TorrentException(ex.getMessage(), ex);
            }
        };
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
