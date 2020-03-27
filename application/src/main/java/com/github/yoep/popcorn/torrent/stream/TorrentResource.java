package com.github.yoep.popcorn.torrent.stream;

import com.github.yoep.popcorn.torrent.models.Torrent;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.FileSystemResource;

import java.io.IOException;
import java.io.InputStream;
import java.lang.ref.WeakReference;

/**
 * Extension on top of {@link FileSystemResource} which returns a {@link TorrentInputStream} instead of the default {@link InputStream}.
 */
@Slf4j
public class TorrentResource extends FileSystemResource {
    private final WeakReference<Torrent> torrent;

    public TorrentResource(Torrent torrent) {
        super(torrent.getVideoFile());
        this.torrent = new WeakReference<>(torrent);
    }

    @Override
    public InputStream getInputStream() throws IOException {
        log.trace("Creating a new input stream for torrent resource {}", torrent);
        var torrent = this.torrent.get();

        if (torrent != null) {
            return new TorrentInputStream(torrent, torrent.getVideoStream());
        } else {
            return null;
        }
    }
}
