package com.github.yoep.torrent.stream.models;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.adapters.torrent.InvalidTorrentStreamStateException;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentException;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.TorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.TorrentStreamListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentState;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentStreamState;
import com.github.yoep.popcorn.backend.torrent.TorrentStreamEventCallback;
import com.github.yoep.popcorn.backend.torrent.TorrentStreamWrapper;
import javafx.beans.property.ReadOnlyObjectProperty;
import javafx.beans.property.ReadOnlyObjectWrapper;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.Resource;
import org.springframework.util.Assert;

import java.io.File;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

@Slf4j
@ToString(exclude = {"listeners"})
@EqualsAndHashCode(exclude = {"listeners"}, callSuper = false)
public class TorrentStreamImpl implements TorrentStream {
    public static final String STATE_PROPERTY = "streamState";

    private final Torrent torrent;
    private final TorrentStreamWrapper wrapper;

    private final ReadOnlyObjectWrapper<TorrentStreamState> streamState = new ReadOnlyObjectWrapper<>(this, STATE_PROPERTY, TorrentStreamState.PREPARING);
    private final List<TorrentStreamListener> listeners = new ArrayList<>();
    private final TorrentStreamEventCallback streamCallback = createStreamCallback();

    //region Constructors

    public TorrentStreamImpl(TorrentStreamWrapper wrapper, Torrent torrent) {
        Assert.notNull(wrapper, "wrapper cannot be null");
        Assert.notNull(torrent, "torrent cannot be null");
        this.wrapper = wrapper;
        this.torrent = torrent;
        FxLib.INSTANCE.register_torrent_stream_callback(wrapper, streamCallback);
        initialize();
    }

    //endregion

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

    //region TorrentStream

    @Override
    public TorrentStreamState getStreamState() {
        return streamState.get();
    }

    @Override
    public ReadOnlyObjectProperty<TorrentStreamState> streamStateProperty() {
        return streamState.getReadOnlyProperty();
    }

    @Override
    public Torrent getTorrent() {
        return torrent;
    }

    @Override
    public String getStreamUrl() {
        return wrapper.getUrl();
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
    public Resource stream() {
        var state = getStreamState();

        // verify if the stream has not been stopped
        if (state == TorrentStreamState.STOPPED) {
            throw new InvalidTorrentStreamStateException(state);
        }

        return new TorrentResource(this);
    }

    @Override
    public void stopStream() {
        pause();
        streamState.set(TorrentStreamState.STOPPED);
    }

    //endregion

    //region Functions

    private void initialize() {
        initializeStateListener();
    }

    private TorrentStreamEventCallback createStreamCallback() {
        return event -> {
            switch (event.getTag()) {
                case StateChanged -> {
                    var change = event.getUnion().getState_changed();
                    streamState.set(change.getNewState());
                }
            }
        };
    }

    private void initializeStateListener() {
        streamState.addListener((observable, oldValue, newValue) -> {
            log.debug("Torrent stream \"{}\" changed from state {} to {}", getFilename(), oldValue, newValue);
            listeners.forEach(e -> safeInvoke(() -> e.onStateChanged(oldValue, newValue)));

            switch (newValue) {
                case STREAMING -> listeners.forEach(e -> safeInvoke(e::onStreamReady));
                case STOPPED -> listeners.forEach(e -> safeInvoke(e::onStreamStopped));
            }
        });
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
