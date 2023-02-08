package com.github.yoep.popcorn.backend.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentException;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.TorrentListener;
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
@Structure.FieldOrder({"filepath", "hasByteCallback"})
public class TorrentWrapper extends Structure implements Torrent, Closeable {

    public String filepath;
    public TorrentHasByteCallback hasByteCallback;

    private final Torrent torrent;

    private TorrentWrapper(Torrent torrent) {
        this.torrent = torrent;
        this.filepath = torrent.getFile().getAbsolutePath();
        this.hasByteCallback = (len, ptr) -> {
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
    public void prioritizePieces(Integer... pieceIndexes) {
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

    public static TorrentWrapper from(Torrent torrent) {
        Objects.requireNonNull(torrent, "torrent cannot be null");
        return new TorrentWrapper(torrent);
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }
}
