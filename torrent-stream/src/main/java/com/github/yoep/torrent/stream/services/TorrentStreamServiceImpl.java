package com.github.yoep.torrent.stream.services;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentException;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import com.github.yoep.popcorn.backend.utils.HostUtils;
import com.github.yoep.torrent.stream.models.TorrentStreamImpl;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;
import org.springframework.boot.autoconfigure.web.ServerProperties;
import org.springframework.util.Assert;
import org.springframework.web.util.UriComponentsBuilder;

import java.util.*;

@Slf4j
@RequiredArgsConstructor
public class TorrentStreamServiceImpl implements TorrentStreamService {
    private final TorrentService torrentService;
    private final ServerProperties serverProperties;
    private final Map<String, TorrentStream> streamCache = new HashMap<>();

    //region TorrentStreamService

    @Override
    public TorrentStream startStream(Torrent torrent) {
        Assert.notNull(torrent, "torrent cannot be null");
        log.trace("Starting a new stream for torrent file {}", torrent.getFile());
        var filename = getFilename(torrent);
        var url = UriComponentsBuilder.newInstance()
                .scheme("http")
                .host(HostUtils.hostAddress())
                .port(serverProperties.getPort())
                .path("/video/{filename}")
                .build(Collections.singletonMap("filename", filename))
                .toString();
        var torrentStream = new TorrentStreamImpl(torrent, url);

        log.debug("Starting stream for torrent {} at {}", filename, url);
        streamCache.put(filename, torrentStream);

        return torrentStream;
    }

    @Override
    public void stopStream(TorrentStream torrentStream) {
        Assert.notNull(torrentStream, "torrentStream cannot be null");
        try {
            var key = getFilename(torrentStream);

            if (streamCache.containsKey(key)) {
                log.debug("Stopping torrentStream stream for {}", key);
                var stream = streamCache.get(key);

                stream.stopStream();
                streamCache.remove(key);
                torrentService.remove(torrentStream.getTorrent());
            } else {
                log.warn("Unable to stop torrentStream stream, torrentStream is unknown ({})", torrentStream);
            }
        } catch (TorrentException ex) {
            log.error("Failed to stop torrent stream, {}", ex.getMessage(), ex);
        }
    }

    @Override
    public void stopAllStreams() {
        var streams = new ArrayList<>(streamCache.values());

        streams.forEach(this::stopStream);
    }

    @Override
    public Optional<TorrentStream> resolve(String filename) {
        Assert.notNull(filename, "filename cannot be null");

        log.trace("Resolving torrent filename {}", filename);
        if (streamCache.containsKey(filename)) {
            return Optional.of(streamCache.get(filename));
        }

        log.warn("Torrent couldn't be found for filename {}", filename);
        return Optional.empty();
    }

    //endregion

    //region Functions

    private String getFilename(Torrent torrent) {
        var filePath = torrent.getFile().getAbsolutePath();

        return FilenameUtils.getName(filePath);
    }

    //endregion

}
