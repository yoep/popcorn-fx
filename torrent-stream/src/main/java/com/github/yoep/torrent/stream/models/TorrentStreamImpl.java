package com.github.yoep.torrent.stream.models;

import com.github.yoep.torrent.adapter.InvalidStreamStateException;
import com.github.yoep.torrent.adapter.TorrentException;
import com.github.yoep.torrent.adapter.listeners.AbstractTorrentListener;
import com.github.yoep.torrent.adapter.listeners.TorrentListener;
import com.github.yoep.torrent.adapter.listeners.TorrentStreamListener;
import com.github.yoep.torrent.adapter.model.Torrent;
import com.github.yoep.torrent.adapter.model.TorrentStream;
import com.github.yoep.torrent.adapter.state.TorrentState;
import com.github.yoep.torrent.adapter.state.TorrentStreamState;
import javafx.beans.property.ReadOnlyObjectProperty;
import javafx.beans.property.ReadOnlyObjectWrapper;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.Resource;
import org.springframework.util.Assert;

import java.io.File;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Optional;

@Slf4j
@ToString
@EqualsAndHashCode
public class TorrentStreamImpl implements TorrentStream {
    public static final String STATE_PROPERTY = "streamState";

    private final Torrent torrent;
    private final String streamUrl;

    private final ReadOnlyObjectWrapper<TorrentStreamState> streamState = new ReadOnlyObjectWrapper<>(this, STATE_PROPERTY, TorrentStreamState.PREPARING);
    private final TorrentListener torrentListener = createTorrentListener();
    private final List<TorrentStreamListener> listeners = new ArrayList<>();
    private final Integer[] preparePieces;

    //region Constructors

    public TorrentStreamImpl(Torrent torrent, String streamUrl) {
        Assert.notNull(torrent, "torrent cannot be null");
        Assert.hasText(streamUrl, "streamUrl cannot be null");
        this.torrent = torrent;
        this.streamUrl = streamUrl;
        this.preparePieces = determinePreparationPieces();
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
        return streamUrl;
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
            throw new InvalidStreamStateException(state);
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
        initializeTorrent();
        updatePiecePriorities();
    }

    private void initializeStateListener() {
        streamState.addListener((observable, oldValue, newValue) -> {
            log.debug("Torrent stream \"{}\" changed from state {} to {}", getFilename(), oldValue, newValue);
            listeners.forEach(e -> safeInvoke(() -> e.onStateChanged(oldValue, newValue)));

            switch (newValue) {
                case STREAMING:
                    listeners.forEach(e -> safeInvoke(e::onStreamReady));
                    break;
                case STOPPED:
                    listeners.forEach(e -> safeInvoke(e::onStreamStopped));
                    break;
            }
        });
    }

    private void initializeTorrent() {
        torrent.addListener(torrentListener);
    }

    private void safeInvoke(Runnable runnable) {
        try {
            runnable.run();
        } catch (Exception ex) {
            log.error("An error occurred while invoking a listener, " + ex.getMessage(), ex);
        }
    }

    private void updatePiecePriorities() {
        log.debug("Preparing the following pieces {} for torrent stream \"{}\"", preparePieces, getFilename());
        // update the torrent file priorities to prepare the first 5 pieces and the last piece
        prioritizePieces(preparePieces);
    }

    private Integer[] determinePreparationPieces() {
        var totalPieces = getTotalPieces();
        var pieces = new ArrayList<Integer>();

        // prepare the first 8 pieces if it doesn't exceed the total pieces
        for (int i = 0; i < 8 && i < totalPieces - 1; i++) {
            pieces.add(i);
        }

        // add the last 2 pieces for preparation
        // this is done for determining the video length during streaming
        pieces.add(totalPieces - 1);
        pieces.add(totalPieces);

        return pieces
                .stream()
                .filter(this::isValidPreparationPiece)
                .toArray(Integer[]::new);
    }

    private boolean isValidPreparationPiece(Integer index) {
        var totalPieces = getTotalPieces();
        var isValid = index >= 0 && index <= totalPieces;

        if (!isValid) {
            log.warn("Preparation piece index {} is invalid", index);
        }

        return isValid;
    }

    private void onPieceFinished() {
        verifyPrepareState();
    }

    private void onTorrentStateChanged() {
        verifyPrepareState();
    }

    private void verifyPrepareState() {
        // check if the torrent is already streaming or stopped
        // if so, ignore this piece finished event
        if (getStreamState() != TorrentStreamState.PREPARING)
            return;

        // verify if all prepare pieces are present
        var preparationCompleted = Arrays.stream(preparePieces)
                .allMatch(this::hasPiece);

        if (preparationCompleted) {
            updateStateToStreaming();
        }
    }

    private void updateStateToStreaming() {
        // verify if the torrent stream isn't already streaming
        // if so, ignore this action
        if (getStreamState() == TorrentStreamState.STREAMING)
            return;

        log.info("Torrent stream \"{}\" is ready to be streamed", getFilename());
        streamState.set(TorrentStreamState.STREAMING);
        // update the torrent download to sequential mode
        sequentialMode();
    }

    private TorrentListener createTorrentListener() {
        return new AbstractTorrentListener() {
            @Override
            public void onStateChanged(TorrentState oldState, TorrentState newState) {
                TorrentStreamImpl.this.onTorrentStateChanged();
            }

            @Override
            public void onPieceFinished(int pieceIndex) {
                TorrentStreamImpl.this.onPieceFinished();
            }
        };
    }

    //endregion
}
