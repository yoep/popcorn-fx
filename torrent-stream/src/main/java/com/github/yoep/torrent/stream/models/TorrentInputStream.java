package com.github.yoep.torrent.stream.models;


import com.github.yoep.popcorn.backend.adapters.torrent.TorrentException;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.AbstractTorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.TorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.io.*;

/**
 * Extension on top of {@link InputStream} which blocks the stream reading when the requested bytes are not yet available.
 */
@Slf4j
@ToString(exclude = "torrentListener")
public class TorrentInputStream extends FilterInputStream {
    private final TorrentListener torrentListener = createTorrentListener();
    private final Torrent torrent;

    private boolean stopped;
    private long location;

    //region Constructors

    /**
     * Initialize a new instance of {@link TorrentInputStream}.
     *
     * @param torrent The torrent this input stream provides.
     * @param file    The torrent file to use in this input stream.
     */
    TorrentInputStream(Torrent torrent, File file) throws FileNotFoundException {
        super(new FileInputStream(file));
        Assert.notNull(torrent, "torrent cannot be null");
        this.torrent = torrent;
        this.torrent.addListener(torrentListener);
    }

    //endregion

    //region FilterInputStream

    @Override
    public synchronized int read() throws IOException {
        if (!waitForPiece(location))
            return -1;

        location++;

        return super.read();
    }

    @Override
    public synchronized int read(byte[] buffer, int offset, int length) throws IOException {
        var pieceLength = torrent.getPieceLength();

        for (int i = 0; i < length; i += pieceLength) {
            if (!waitForPiece(location + i)) {
                return -1;
            }
        }

        location += length;

        return super.read(buffer, offset, length);
    }

    @Override
    public void close() throws IOException {
        log.trace("Closing torrent input stream {}", this);
        synchronized (this) {
            stopped = true;
            notifyAll();
        }

        this.torrent.removeListener(torrentListener);

        super.close();
    }

    @Override
    public synchronized long skip(long length) throws IOException {
        log.trace("Skipping {} bytes in torrent input stream {}", length, this);
        location += length;
        return super.skip(length);
    }

    @Override
    public boolean markSupported() {
        return false;
    }

    //endregion

    //region Functions

    private synchronized boolean waitForPiece(long offset) {
        while (!Thread.currentThread().isInterrupted() && !stopped) {
            try {
                // check if the byte is already downloaded
                if (torrent.hasByte(offset)) {
                    return true;
                }

                // prioritize the byte download
                torrent.prioritizeByte(offset);

                log.trace("Waiting for offset {} to be present in torrent input stream {}", offset, this);
                this.wait();
                log.trace("Continuing the torrent input stream thread {}", this);
            } catch (InterruptedException ex) {
                log.debug("Torrent input stream wait got interrupted for {}", this);
                Thread.currentThread().interrupt();
            } catch (TorrentException ex) {
                log.error(ex.getMessage(), ex);
                return false;
            } catch (Exception ex) {
                log.error(ex.getMessage(), ex);
            }
        }

        return false;
    }

    private void pieceFinished() {
        synchronized (this) {
            log.trace("Awakening the torrent input stream thread");
            this.notifyAll();
        }
    }

    private TorrentListener createTorrentListener() {
        return new AbstractTorrentListener() {
            @Override
            public void onPieceFinished(int pieceIndex) {
                pieceFinished();
            }
        };
    }

    //endregion
}
