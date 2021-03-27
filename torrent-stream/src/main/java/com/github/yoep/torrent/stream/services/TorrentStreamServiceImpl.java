package com.github.yoep.torrent.stream.services;

import com.github.yoep.torrent.adapter.StreamException;
import com.github.yoep.torrent.adapter.TorrentService;
import com.github.yoep.torrent.adapter.TorrentStreamService;
import com.github.yoep.torrent.adapter.model.Torrent;
import com.github.yoep.torrent.adapter.model.TorrentStream;
import com.github.yoep.torrent.stream.models.TorrentStreamImpl;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;
import org.springframework.context.ApplicationContext;
import org.springframework.util.Assert;
import org.springframework.util.StringUtils;
import org.springframework.web.util.UriComponentsBuilder;

import java.text.MessageFormat;
import java.util.*;

@Slf4j
public class TorrentStreamServiceImpl implements TorrentStreamService {
    static final String PORT_PROPERTY = "server.port";

    private final TorrentService torrentService;
    private final ApplicationContext applicationContext;
    private final Map<String, TorrentStream> streamCache = new HashMap<>();
    private final int port;

    //region Constructors

    public TorrentStreamServiceImpl(TorrentService torrentService, ApplicationContext applicationContext) {
        this.torrentService = torrentService;
        this.applicationContext = applicationContext;
        this.port = getPort();
    }

    //endregion

    //region TorrentStreamService

    @Override
    public TorrentStream startStream(Torrent torrent) {
        Assert.notNull(torrent, "torrent cannot be null");
        var filename = getFilename(torrent);
        var url = UriComponentsBuilder.newInstance()
                .scheme("http")
                .host("127.0.0.1")
                .port(port)
                .path("/video/{filename}")
                .build(Collections.singletonMap("filename", filename))
                .toString();
        var torrentStream = new TorrentStreamImpl(torrent, url);

        log.debug("Starting stream for torrent {}", filename);
        streamCache.put(filename, torrentStream);

        return torrentStream;
    }

    @Override
    public void stopStream(TorrentStream torrentStream) {
        Assert.notNull(torrentStream, "torrentStream cannot be null");
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

    private int getPort() {
        var environment = applicationContext.getEnvironment();
        var portValue = environment.getProperty(PORT_PROPERTY);

        // verify if the port could be found
        if (!StringUtils.hasText(portValue)) {
            throw new StreamException(MessageFormat.format("Unable to determine port, port property \"{0}\" is null or empty", PORT_PROPERTY));
        }

        return Integer.parseInt(portValue);
    }

    private String getFilename(Torrent torrent) {
        var filePath = torrent.getFile().getAbsolutePath();

        return FilenameUtils.getName(filePath);
    }

    //endregion

}
