package com.github.yoep.torrent.frostwire.model;

import com.frostwire.jlibtorrent.*;
import com.frostwire.jlibtorrent.alerts.Alert;
import com.frostwire.jlibtorrent.alerts.AlertType;
import com.frostwire.jlibtorrent.alerts.MetadataFailedAlert;
import com.frostwire.jlibtorrent.alerts.PieceFinishedAlert;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentException;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.TorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentState;
import javafx.beans.property.ReadOnlyObjectProperty;
import javafx.beans.property.ReadOnlyObjectWrapper;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.io.File;
import java.text.MessageFormat;
import java.util.Arrays;
import java.util.Optional;
import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedQueue;

@Slf4j
@ToString(exclude = "handle")
@EqualsAndHashCode(exclude = "handle")
public class FrostTorrent implements Torrent, AlertListener {
    public static final String STATE_PROPERTY = "state";

    private final ReadOnlyObjectWrapper<TorrentState> state = new ReadOnlyObjectWrapper<>(this, STATE_PROPERTY, TorrentState.CREATING);
    private final Queue<TorrentListener> listeners = new ConcurrentLinkedQueue<>();
    private final TorrentPieces pieces = new TorrentPieces();
    private final TorrentHandle handle;
    private final String filename;
    private final int fileIndex;
    private final boolean autoStartDownload;

    private TorrentException error;

    //region Constructors

    public FrostTorrent(TorrentHandle handle, int fileIndex, boolean autoStartDownload) {
        Assert.notNull(handle, "handle cannot be null");
        Assert.isTrue(fileIndex >= 0, "fileIndex cannot be smaller than 0");
        this.handle = handle;
        this.fileIndex = fileIndex;
        this.filename = handle.torrentFile().files().fileName(fileIndex);
        this.autoStartDownload = autoStartDownload;

        init();
    }

    //endregion

    //region Torrent

    @Override
    public TorrentState getState() {
        return state.get();
    }

    @Override
    public ReadOnlyObjectProperty<TorrentState> stateProperty() {
        return state.getReadOnlyProperty();
    }

    @Override
    public Optional<TorrentException> getError() {
        return Optional.ofNullable(error);
    }

    @Override
    public String getFilename() {
        return filename;
    }

    @Override
    public File getFile() {
        if (!handle.isValid()) {
            throw new TorrentException("Failed to get file, torrent handle has been invalidated");
        }

        var files = handle.torrentFile().files();
        var savePath = handle.savePath();
        var filePath = files.filePath(fileIndex);

        return new File(savePath + "/" + filePath);
    }

    @Override
    public Integer getPieceLength() {
        var torrentInfo = handle.torrentFile();

        return torrentInfo.pieceLength();
    }

    @Override
    public boolean hasPiece(int pieceIndex) {
        return handle.havePiece(pieceIndex + pieces.getFirstPieceIndex());
    }

    @Override
    public int getTotalPieces() {
        return pieces.getLastPieceIndex() - pieces.getFirstPieceIndex();
    }

    @Override
    public void prioritizePieces(int... pieceIndexes) {
        log.trace("Prioritizing the following pieces: {}", Arrays.toString(pieceIndexes));
        for (int pieceIndex : pieceIndexes) {
            var torrentPieceIndex = pieces.getTorrentPieceIndex(pieceIndex);

            // verify if the torrent piece index is within the download range
            if (pieces.isInDownloadRange(torrentPieceIndex)) {
                prioritizePiece(torrentPieceIndex, true);
            } else {
                log.error("Torrent piece {} cannot be prioritized as it's not within the torrent download range [{}-{}]",
                        torrentPieceIndex, pieces.getFirstPieceIndex(), pieces.getLastPieceIndex());
            }
        }
    }

    @Override
    public boolean hasByte(long byteIndex) {
        if (handle.isValid()) {
            int pieceIndex = getPieceIndexOfByte(byteIndex);

            return handle.havePiece(pieceIndex);
        } else {
            throw new TorrentException("Handle is invalid for " + filename);
        }
    }

    @Override
    public void prioritizeBytes(long... bytes) {
        var indexes = Arrays.stream(bytes)
                .mapToInt(this::getPieceIndexOfByte)
                .distinct()
                .toArray();

        for (var pieceIndex : indexes) {
            var nextPiece = pieceIndex + 1;

            // prioritize the piece of the byte
            prioritizePiece(pieceIndex, false);

            // prioritize the next piece if it's within the current download range
            // this is done to prevent stream tearing
            if (nextPiece <= pieces.getLastPieceIndex()) {
                prioritizePiece(nextPiece, false);
            }
        }
    }

    @Override
    public void addListener(TorrentListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.add(listener);
        }
    }

    @Override
    public void removeListener(TorrentListener listener) {
        synchronized (listeners) {
            listeners.remove(listener);
        }
    }

    @Override
    public void startDownload() {
        if (getState() != TorrentState.READY)
            return;

        updateState(TorrentState.STARTING);
        resume();
    }

    @Override
    public void resume() {
        if (getState() == TorrentState.COMPLETED)
            return;

        handle.resume();
    }

    @Override
    public void pause() {
        if (getState() == TorrentState.COMPLETED)
            return;

        updateState(TorrentState.PAUSED);

        if (handle.isValid()) {
            handle.pause();
        } else {
            log.warn("Unable to pause torrent {}, torrent handle is not valid anymore", filename);
        }
    }

    @Override
    public void sequentialMode() {
        log.debug("Updating torrent download \"{}\" to sequential mode", filename);
        handle.clearPieceDeadlines();
        handle.setFlags(handle.flags().and_(TorrentFlags.SEQUENTIAL_DOWNLOAD));
    }

    //endregion

    //region Getters

    public TorrentHandle getHandle() {
        return handle;
    }

    //endregion

    //region AlertListener

    @Override
    public int[] types() {
        return new int[]{
                AlertType.METADATA_FAILED.swig(),
                AlertType.STATS.swig(),
                AlertType.PIECE_FINISHED.swig()
        };
    }

    @Override
    public void alert(Alert<?> alert) {
        try {
            switch (alert.type()) {
                case METADATA_FAILED:
                    onMetadataFailed(alert);
                    break;
                case STATS:
                    onStatsReceived();
                    break;
                case PIECE_FINISHED:
                    onPieceFinished(alert);
                    break;
            }
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    //endregion

    //region Functions

    private void init() {
        initializeStateListener();

        try {
            initializeFilePriorities();
            initializePieces();
            initializeAutoStart();
        } catch (Exception ex) {
            handleInitializationFailure(ex);
        }
    }

    private void initializeStateListener() {
        state.addListener((observable, oldValue, newValue) -> {
            log.debug("Torrent \"{}\" changed from state {} to {}", getFilename(), oldValue, newValue);
            synchronized (listeners) {
                listeners.forEach(e -> safeInvoke(() -> e.onStateChanged(oldValue, newValue)));

                if (newValue == TorrentState.ERROR) {
                    listeners.forEach(e -> safeInvoke(() -> e.onError(error)));
                }
            }
        });
    }

    private void initializeFilePriorities() {
        log.trace("Preparing torrent \"{}\" file priorities for file index {}", filename, fileIndex);
        var torrentInfo = handle.torrentFile();
        var filePriorities = new Priority[torrentInfo.numFiles()];

        // update the torrent files so that only file index is downloaded
        for (int i = 0; i < torrentInfo.numFiles(); i++) {
            if (i == fileIndex) {
                filePriorities[i] = Priority.NORMAL;
            } else {
                filePriorities[i] = Priority.IGNORE;
            }
        }

        handle.prioritizeFiles(filePriorities);

        // wait for jlibtorrent to reflect the change
        // as of a newer update, the TorrentHandle#prioritizeFiles doesn't wait for thew change to apply
        // which causes issues when prioritizing pieces
        while (!Arrays.equals(handle.filePriorities(), filePriorities)) {
            Thread.onSpinWait();
        }
    }

    private void initializePieces() {
        var priorities = handle.piecePriorities();

        pieces.determineDownloadPieceIndexes(priorities);
        updateState(TorrentState.READY);
    }

    private void initializeAutoStart() {
        if (autoStartDownload) {
            log.trace("Auto starting the download for torrent \"{}\"", filename);
            startDownload();
        }
    }

    private void handleInitializationFailure(Exception ex) {
        var message = MessageFormat.format("Torrent \"{0}\" initialization failed, {1}", filename, ex.getMessage());
        log.error(message, ex);

        if (TorrentException.class.isAssignableFrom(ex.getClass())) {
            error = (TorrentException) ex;
        } else {
            error = new TorrentException(ex.getMessage(), ex);
        }

        state.set(TorrentState.ERROR);
    }

    private void onMetadataFailed(Alert<?> alert) {
        var metadataFailedAlert = (MetadataFailedAlert) alert;
        var error = metadataFailedAlert.getError();
        var message = MessageFormat.format("Torrent \"{0}\" encountered an error while retrieving the metadata, code: {1} - {2}",
                filename, error.value(), error.message());

        this.error = new TorrentException(message);

        log.error(this.error.getMessage(), this.error);
        state.set(TorrentState.ERROR);
    }

    private void onPieceFinished(Alert<?> alert) {
        var pieceFinishedAlert = (PieceFinishedAlert) alert;
        var pieceIndex = pieceFinishedAlert.pieceIndex() - pieces.getFirstPieceIndex();

        // notify all listeners
        synchronized (listeners) {
            for (TorrentListener listener : listeners) {
                safeInvoke(() -> listener.onPieceFinished(pieceIndex));
            }
        }

        // check if we need to update the torrent state
        if (getState() != TorrentState.COMPLETED) {
            updateState(TorrentState.DOWNLOADING);
        }
    }

    private void onStatsReceived() {
        if (!handle.isValid())
            return;

        var status = handle.status();
        var state = status.state();
        var downloadStatus = FrostDownloadStatus.builder()
                .progress(status.progress())
                .downloadSpeed(status.downloadPayloadRate())
                .uploadSpeed(status.uploadPayloadRate())
                .seeds(status.numSeeds())
                .peers(status.numPeers())
                .downloaded(status.totalWantedDone())
                .totalSize(status.totalWanted())
                .build();

        // check if the torrent state is finished
        // if so, update this torrent state to completed
        if (state == TorrentStatus.State.FINISHED || state == TorrentStatus.State.SEEDING) {
            updateState(TorrentState.COMPLETED);
        }

        synchronized (listeners) {
            listeners.forEach(e -> safeInvoke(() -> e.onDownloadStatus(downloadStatus)));
        }
    }

    private void safeInvoke(Runnable runnable) {
        try {
            runnable.run();
        } catch (Exception ex) {
            log.error("An error occurred while invoking a listener, " + ex.getMessage(), ex);
        }
    }

    private void prioritizePiece(int pieceIndex, boolean useDeadline) {
        handle.piecePriority(pieceIndex, Priority.SEVEN);

        if (useDeadline) {
            handle.setPieceDeadline(pieceIndex, 1000);
        }
    }

    private void updateState(TorrentState newState) {
        // check if the torrent was previously in an error state
        // if so, we refuse the torrent state to be updated
        if (getState() == TorrentState.ERROR) {
            log.debug("Unable to update torrent state to \"{}\", current state is \"{}\"", newState, getState());
            return;
        }

        state.set(newState);
    }

    /**
     * Get the piece index of the byte within the full torrent.
     * This method automatically adds the firstPieceIndex to the index of the byte in the fileIndex.
     *
     * @param byteIndex The file byte index.
     * @return Returns the piece index of the byte in regards to the full torrent pieces.
     */
    private int getPieceIndexOfByte(long byteIndex) {
        var torrentInfo = handle.torrentFile();
        var pieceIndex = (int) (byteIndex / torrentInfo.pieceLength());

        return pieces.getTorrentPieceIndex(pieceIndex);
    }

    //endregion
}
