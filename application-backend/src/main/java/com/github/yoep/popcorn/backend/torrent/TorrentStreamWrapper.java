package com.github.yoep.popcorn.backend.torrent;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentException;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.TorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.TorrentStreamListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentState;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentStreamState;
import com.github.yoep.popcorn.backend.lib.FxLibInstance;
import com.sun.jna.Pointer;
import com.sun.jna.Structure;
import javafx.beans.property.ReadOnlyObjectProperty;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.io.Closeable;
import java.io.File;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

@Slf4j
@Getter
@ToString
@EqualsAndHashCode(exclude = {"listeners", "streamCallback"}, callSuper = false)
@Structure.FieldOrder({"streamUrl", "ptr"})
public class TorrentStreamWrapper extends Structure implements Closeable, TorrentStream {
    public String streamUrl;
    public Pointer ptr;
    private Torrent torrent;
    private boolean callbackRegistered;

    private final List<TorrentStreamListener> listeners = new ArrayList<>();
    private final TorrentStreamEventCallback streamCallback = createStreamCallback();

    public TorrentStreamWrapper() {
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
    public void prioritizeBytes(long... bytes) {
        torrent.prioritizeBytes(bytes);
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

    //region TorrentStream

    @Override
    public TorrentStreamState streamState() {
        return FxLibInstance.INSTANCE.get().torrent_stream_state(this);
    }

    @Override
    public Torrent getTorrent() {
        return torrent;
    }

    @Override
    public void addListener(TorrentStreamListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        listeners.add(listener);
    }

    @Override
    public void removeListener(TorrentStreamListener listener) {
        listeners.remove(listener);
    }

    @Override
    public void stopStream() {
        pause();
    }

    //endregion

    public void updateTorrent(Torrent torrent) {
        if (this.torrent != null)
            return;

        this.torrent = torrent;
    }

    @Override
    public void read() {
        super.read();

        if (!callbackRegistered) {
            callbackRegistered = true;
            FxLibInstance.INSTANCE.get().register_torrent_stream_callback(this, streamCallback);
        }
    }

    @Override
    public void close() {
        setAutoSynch(false);
    }

    //region Functions

    private TorrentStreamEventCallback createStreamCallback() {
        return event -> {
            switch (event.getTag()) {
                case StateChanged -> {
                    var change = event.getUnion().getState_changed();
                    var newState = change.getNewState();

                    listeners.forEach(e -> safeInvoke(() -> e.onStateChanged(newState)));

                    switch (newState) {
                        case STREAMING -> listeners.forEach(e -> safeInvoke(e::onStreamReady));
                        case STOPPED -> listeners.forEach(e -> safeInvoke(e::onStreamStopped));
                    }
                }
            }
        };
    }

    private void safeInvoke(Runnable runnable) {
        try {
            runnable.run();
        } catch (Exception ex) {
            log.error("An error occurred while invoking a listener, " + ex.getMessage(), ex);
        }
    }

    //endregion
}
