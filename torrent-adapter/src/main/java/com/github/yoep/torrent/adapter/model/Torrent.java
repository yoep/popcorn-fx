package com.github.yoep.torrent.adapter.model;

import com.github.yoep.torrent.adapter.TorrentException;
import com.github.yoep.torrent.adapter.listeners.TorrentListener;
import com.github.yoep.torrent.adapter.state.TorrentState;
import javafx.beans.property.ReadOnlyObjectProperty;

import java.io.File;
import java.util.Optional;

public interface Torrent {

    /**
     * Get the state of the torrent.
     *
     * @return Returns the current state of the torrent.
     */
    TorrentState getState();

    /**
     * Get the state property of this torrent.
     *
     * @return Returns the state property.
     */
    ReadOnlyObjectProperty<TorrentState> stateProperty();

    /**
     * Get the error that occurred in the torrent.
     * An error will only be present if the {@link #getState()} is {@link TorrentState#ERROR}.
     *
     * @return Returns the error that occurred.
     */
    Optional<TorrentException> getError();

    /**
     * Get the filename of the torrent.
     *
     * @return Returns the filename of the torrent.
     */
    String getFilename();

    /**
     * Get the file of this torrent.
     *
     * @return Returns the file of this torrent.
     */
    File getFile();

    /**
     * Get the length of each piece in the torrent.
     *
     * @return Returns the length of each torrent piece.
     */
    Integer getPieceLength();

    /**
     * Verify if the torrent has the given piece.
     *
     * @param pieceIndex The piece index to verify.
     * @return Returns true if the piece has already been downloaded, else false.
     */
    boolean hasPiece(int pieceIndex);

    /**
     * Get the total number of pieces that will be downloaded for this torrent.
     *
     * @return Returns the total number of pieces.
     */
    int getTotalPieces();

    /**
     * Prioritize the given piece indexes for downloading.
     *
     * @param pieceIndexes The piece indexes to prioritize.
     */
    void prioritizePieces(Integer... pieceIndexes);

    /**
     * Verify if the torrent has the given bytes.
     *
     * @param byteIndex The byte index.
     * @return Returns true if the given byte is downloaded, else false.
     */
    boolean hasByte(long byteIndex);

    /**
     * Prioritize the given bytes in the download.
     * This will increase the priority of the piece index which contains the byteIndex.
     *
     * @param byteIndex The byte index to prioritize.
     */
    void prioritizeByte(long byteIndex);

    /**
     * Register a new listener for this torrent.
     *
     * @param listener The listener to register.
     */
    void addListener(TorrentListener listener);

    /**
     * Remove a registered listener from this torrent.
     *
     * @param listener The listener to remove.
     */
    void removeListener(TorrentListener listener);

    /**
     * Start downloading the torrent file.
     * This action will be ignored if the torrent download has already been started.
     */
    void startDownload();

    /**
     * Resume the torrent download.
     * This action will be ignored if {@link #getState()} is {@link TorrentState#COMPLETED}.
     */
    void resume();

    /**
     * Pause the torrent download.
     * This action will be ignored if {@link #getState()} is {@link TorrentState#COMPLETED}.
     */
    void pause();

    /**
     * Update the torrent download mode to sequential.
     * This mode is important for when the torrent is being streamed.
     */
    void sequentialMode();
}
