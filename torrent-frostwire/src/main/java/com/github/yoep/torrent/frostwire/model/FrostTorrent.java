package com.github.yoep.torrent.frostwire.model;

import com.frostwire.jlibtorrent.AlertListener;
import com.frostwire.jlibtorrent.Priority;
import com.frostwire.jlibtorrent.TorrentFlags;
import com.frostwire.jlibtorrent.TorrentHandle;
import com.frostwire.jlibtorrent.alerts.Alert;
import com.frostwire.jlibtorrent.alerts.AlertType;
import com.frostwire.jlibtorrent.alerts.PieceFinishedAlert;
import com.github.yoep.torrent.adapter.TorrentException;
import com.github.yoep.torrent.adapter.listeners.TorrentListener;
import com.github.yoep.torrent.adapter.model.Torrent;
import com.github.yoep.torrent.adapter.state.TorrentState;
import javafx.beans.property.ReadOnlyObjectProperty;
import javafx.beans.property.ReadOnlyObjectWrapper;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.io.File;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

@Slf4j
@ToString(exclude = "handle")
@EqualsAndHashCode(exclude = "handle")
public class FrostTorrent implements Torrent, AlertListener {
    public static final String STATE_PROPERTY = "state";

    private final ReadOnlyObjectWrapper<TorrentState> state = new ReadOnlyObjectWrapper<>(this, STATE_PROPERTY, TorrentState.CREATING);
    private final List<TorrentListener> listeners = new ArrayList<>();
    private final TorrentHandle handle;
    private final String filename;
    private final int fileIndex;
    private final int firstPieceIndex;
    private final int lastPieceIndex;

    //region Constructors

    public FrostTorrent(TorrentHandle handle, int fileIndex, boolean startDownload) {
        Assert.notNull(handle, "handle cannot be null");
        Assert.isTrue(fileIndex >= 0, "fileIndex cannot be smaller than 0");
        this.handle = handle;
        this.fileIndex = fileIndex;
        this.filename = handle.torrentFile().files().fileName(fileIndex);

        initialize();

        this.firstPieceIndex = getFirstPieceIndex();
        this.lastPieceIndex = getLastPieceIndex();

        if (startDownload) {
            startDownload();
        }
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
    public String getFilename() {
        return filename;
    }

    @Override
    public File getFile() {
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
        return handle.havePiece(pieceIndex + firstPieceIndex);
    }

    @Override
    public int getTotalPieces() {
        return lastPieceIndex - firstPieceIndex;
    }

    @Override
    public void prioritizePieces(Integer... pieceIndexes) {
        Assert.noNullElements(pieceIndexes, "pieceIndexes cannot contain \"null\" items");
        log.trace("Prioritizing the following pieces: {}", Arrays.toString(pieceIndexes));
        for (int pieceIndex : pieceIndexes) {
            prioritizePiece(pieceIndex + firstPieceIndex, true);
        }
    }

    @Override
    public boolean hasByte(long byteIndex) {
        int pieceIndex = getPieceIndexOfByte(byteIndex);

        return handle.havePiece(pieceIndex);
    }

    @Override
    public void prioritizeByte(long byteIndex) {
        var pieceIndex = getPieceIndexOfByte(byteIndex);
        var nextPiece = pieceIndex + 1;

        // prioritize the piece of the byte
        prioritizePiece(pieceIndex, false);

        // prioritize the next piece if it's within the current download range
        // this is done to prevent stream tearing
        if (nextPiece <= lastPieceIndex) {
            prioritizePiece(nextPiece, false);
        }
    }

    @Override
    public void addListener(TorrentListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        listeners.add(listener);
    }

    @Override
    public void removeListener(TorrentListener listener) {
        listeners.remove(listener);
    }

    @Override
    public void startDownload() {
        if (getState() != TorrentState.CREATING)
            return;

        state.set(TorrentState.STARTING);
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

        state.set(TorrentState.PAUSED);
        handle.pause();
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
                AlertType.STATS.swig(),
                AlertType.PIECE_FINISHED.swig()
        };
    }

    @Override
    public void alert(Alert<?> alert) {
        try {
            switch (alert.type()) {
                case STATS:
                    sendStreamProgress();
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

    private void initialize() {
        initializeStateListener();
        initializeTorrentFilePriorities();
    }

    private void initializeStateListener() {
        state.addListener((observable, oldValue, newValue) -> listeners.forEach(e -> safeInvoke(() -> e.onStateChanged(oldValue, newValue))));
    }

    private void initializeTorrentFilePriorities() {
        log.trace("Preparing torrent");
        var torrentInfo = handle.torrentFile();

        // update the torrent files so that only file index is downloaded
        for (int i = 0; i < torrentInfo.numFiles(); i++) {
            if (i == fileIndex) {
                handle.filePriority(i, Priority.NORMAL);
            } else {
                handle.filePriority(i, Priority.IGNORE);
            }
        }
    }

    private void onPieceFinished(Alert<?> alert) {
        var pieceFinishedAlert = (PieceFinishedAlert) alert;
        var pieceIndex = pieceFinishedAlert.pieceIndex() - firstPieceIndex;
        var downloadComplete = true;

        // notify all listeners
        listeners.forEach(e -> safeInvoke(() -> e.onPieceFinished(pieceIndex)));

        // check if all pieces are downloaded
        // if so, update the state to completed, else to downloading
        for (int i = firstPieceIndex; i <= lastPieceIndex; i++) {
            if (!handle.havePiece(i)) {
                downloadComplete = false;
                break;
            }
        }

        if (downloadComplete) {
            state.set(TorrentState.COMPLETED);
        } else {
            state.set(TorrentState.DOWNLOADING);
        }
    }

    private void sendStreamProgress() {
        var status = handle.status();
        var downloadStatus = FrostDownloadStatus.builder()
                .progress(status.progress())
                .downloadSpeed(status.downloadRate())
                .uploadSpeed(status.uploadRate())
                .seeds(status.numSeeds())
                .downloaded(status.totalWantedDone())
                .totalSize(status.totalWanted())
                .build();


        listeners.forEach(e -> safeInvoke(() -> e.onDownloadProgress(downloadStatus)));
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

        return pieceIndex + firstPieceIndex;
    }

    private int getFirstPieceIndex() {
        var priorities = handle.piecePriorities();

        for (int i = 0; i < priorities.length; i++) {
            var priority = priorities[i];

            if (priority == Priority.NORMAL)
                return i;
        }

        throw new TorrentException("First piece index could not be found");
    }

    private int getLastPieceIndex() {
        var priorities = handle.piecePriorities();

        for (int i = priorities.length - 1; i >= 0; i--) {
            var priority = priorities[i];

            if (priority == Priority.NORMAL)
                return i;
        }

        throw new TorrentException("Last piece index could not be found");
    }

    //endregion
}
