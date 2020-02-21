package com.github.yoep.popcorn.torrent.stream;

import com.github.yoep.popcorn.torrent.models.Torrent;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.FileSystemResource;

import java.io.IOException;
import java.io.InputStream;

/**
 * Extension on top of {@link FileSystemResource} which returns a {@link TorrentInputStream} instead of the default {@link InputStream}.
 */
@Slf4j
public class TorrentResource extends FileSystemResource {
    private final Torrent torrent;

    public TorrentResource(Torrent torrent) {
        super(torrent.getVideoFile());
        this.torrent = torrent;
    }

    @Override
    public InputStream getInputStream() throws IOException {
        log.trace("Creating a new input stream for torrent resource {}", torrent);
        return new TorrentInputStream(torrent, torrent.getVideoStream());
    }
}
