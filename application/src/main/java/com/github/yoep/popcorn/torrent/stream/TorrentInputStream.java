package com.github.yoep.popcorn.torrent.stream;

import com.frostwire.jlibtorrent.AlertListener;
import com.frostwire.jlibtorrent.alerts.Alert;
import com.frostwire.jlibtorrent.alerts.AlertType;
import com.github.yoep.popcorn.torrent.models.Torrent;
import lombok.extern.slf4j.Slf4j;

import java.io.FilterInputStream;
import java.io.IOException;
import java.io.InputStream;

/**
 * Extension on top of {@link InputStream} which blocks the stream reading when the requested bytes are not yet available.
 */
@Slf4j
public class TorrentInputStream extends FilterInputStream implements AlertListener {
    //TODO: fix this cheaty workaround because 2 different torrent input streams are created
    private static final Object monitor = new Object();
    private final Torrent torrent;

    private boolean stopped;
    private long location;

    /**
     * Initialize a new instance of {@link TorrentInputStream}.
     *
     * @param torrent     The torrent this input stream provides.
     * @param inputStream The parent input stream of this filtered stream.
     */
    public TorrentInputStream(Torrent torrent, InputStream inputStream) {
        super(inputStream);
        this.torrent = torrent;
    }

    @Override
    public synchronized int read() throws IOException {
        if (!waitForPiece(location))
            return -1;

        location++;

        return super.read();
    }

    @Override
    public synchronized int read(byte[] buffer, int offset, int length) throws IOException {
        int pieceLength = torrent.getTorrentHandle().torrentFile().pieceLength();

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

    @Override
    public int[] types() {
        return new int[]{
                AlertType.PIECE_FINISHED.swig(),
        };
    }

    @Override
    public void alert(Alert<?> alert) {
        switch (alert.type()) {
            case PIECE_FINISHED:
                pieceFinished();
                break;
            default:
                break;
        }
    }

    private boolean waitForPiece(long offset) {
        synchronized (monitor) {
            while (!Thread.currentThread().isInterrupted() && !stopped) {
                try {
                    if (torrent.hasBytes(offset)) {
                        return true;
                    }

                    log.trace("Waiting for offset {} to be present in torrent input stream {}", offset, this);
                    monitor.wait();
                    log.trace("Continuing the torrent input stream thread {}", this);
                } catch (InterruptedException ex) {
                    log.debug("Torrent input stream wait got interrupted for {}", this);
                    Thread.currentThread().interrupt();
                }
            }
        }

        return false;
    }

    private void pieceFinished() {
        synchronized (monitor) {
            log.trace("Awakening the torrent input stream thread {}", this);
            monitor.notifyAll();
        }
    }
}
