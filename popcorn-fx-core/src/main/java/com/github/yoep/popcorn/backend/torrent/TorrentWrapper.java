package com.github.yoep.popcorn.backend.torrent;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentException;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.TorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentState;
import com.sun.jna.Structure;
import javafx.beans.property.ReadOnlyObjectProperty;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

import java.io.Closeable;
import java.io.File;
import java.util.Objects;
import java.util.Optional;

@Slf4j
@Getter
@ToString
@EqualsAndHashCode(callSuper = false)
@Structure.FieldOrder({"filepath", "hasByteCallback", "hasPieceCallback", "torrentTotalPiecesCallback", "prioritizePiecesCallback", "sequentialModeCallback"})
public class TorrentWrapper extends Structure implements Torrent, Closeable {
    public static class ByValue extends TorrentWrapper implements Structure.ByValue {
        public ByValue(Torrent torrent) {
            super(torrent);
            this.wrapperPointer = FxLib.INSTANCE.torrent_wrapper(this);
            log.trace("Created torrent wrapper pointer {}", this.wrapperPointer);
        }
    }

    public String filepath;
    public TorrentHasByteCallback hasByteCallback;
    public TorrentHasPieceCallback hasPieceCallback;
    public TorrentTotalPiecesCallback torrentTotalPiecesCallback;
    public PrioritizePiecesCallback prioritizePiecesCallback;
    public SequentialModeCallback sequentialModeCallback;

    private final Torrent torrent;
    TorrentWrapperPointer wrapperPointer;

    private TorrentWrapper(Torrent torrent) {
        this.torrent = torrent;
        this.filepath = torrent.getFile().getAbsolutePath();
        this.hasByteCallback = createHasByteCallback();
        this.hasPieceCallback = (int index) -> (byte) (this.torrent.hasPiece(index) ? 1 : 0);
        this.torrentTotalPiecesCallback = torrent::getTotalPieces;
        this.prioritizePiecesCallback = createPrioritizePiecesCallback();
        this.sequentialModeCallback = this.torrent::sequentialMode;
        initialize();
    }

    //region Torrent

    @Override
    public TorrentState getState() {
        return torrent.getState();
    }

    @Override
    public ReadOnlyObjectProperty<TorrentState> stateProperty() {
        return torrent.stateProperty();
    }

    @Override
    public Optional<TorrentException> getError() {
        return torrent.getError();
    }

    @Override
    public String getFilename() {
        return torrent.getFilename();
    }

    @Override
    public File getFile() {
        return torrent.getFile();
    }

    @Override
    public Integer getPieceLength() {
        return torrent.getPieceLength();
    }

    @Override
    public boolean hasPiece(int pieceIndex) {
        return torrent.hasPiece(pieceIndex);
    }

    @Override
    public int getTotalPieces() {
        return torrent.getTotalPieces();
    }

    @Override
    public void prioritizePieces(int... pieceIndexes) {
        torrent.prioritizePieces(pieceIndexes);
    }

    @Override
    public boolean hasByte(long byteIndex) {
        return torrent.hasByte(byteIndex);
    }

    @Override
    public void prioritizeByte(long byteIndex) {
        torrent.prioritizeByte(byteIndex);
    }

    @Override
    public void addListener(TorrentListener listener) {
        torrent.addListener(listener);
    }

    @Override
    public void removeListener(TorrentListener listener) {
        torrent.removeListener(listener);
    }

    @Override
    public void startDownload() {
        torrent.startDownload();
    }

    @Override
    public void resume() {
        torrent.resume();
    }

    @Override
    public void pause() {
        torrent.pause();
    }

    @Override
    public void sequentialMode() {
        torrent.sequentialMode();
    }


    //endregion

    //region Methods

    public static TorrentWrapper.ByValue from(Torrent torrent) {
        Objects.requireNonNull(torrent, "torrent cannot be null");
        return new TorrentWrapper.ByValue(torrent);
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }

    //endregion

    private void initialize() {
        this.torrent.addListener(new TorrentListener() {
            @Override
            public void onStateChanged(TorrentState oldState, TorrentState newState) {
                FxLib.INSTANCE.torrent_state_changed(wrapperPointer, newState);
            }

            @Override
            public void onError(TorrentException error) {

            }

            @Override
            public void onDownloadStatus(DownloadStatus status) {

            }

            @Override
            public void onPieceFinished(int pieceIndex) {
                FxLib.INSTANCE.torrent_piece_finished(wrapperPointer, pieceIndex);
            }
        });
    }

    private TorrentHasByteCallback createHasByteCallback() {
        return (len, ptr) -> {
            if (len == 0 || ptr == null)
                return (byte) 1;

            var bytes = ptr.getLongArray(0, len);
            for (long byteIndex : bytes) {
                if (!torrent.hasByte(byteIndex)) {
                    return (byte) 0;
                }
            }

            return (byte) 1;
        };
    }

    private PrioritizePiecesCallback createPrioritizePiecesCallback() {
        return (len, ptr) -> {
            if (len == 0 || ptr == null)
                return;

            torrent.prioritizePieces(ptr.getIntArray(0, len));
        };
    }
}