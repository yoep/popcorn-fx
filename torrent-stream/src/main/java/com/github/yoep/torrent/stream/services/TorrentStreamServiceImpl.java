package com.github.yoep.torrent.stream.services;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentException;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import com.github.yoep.popcorn.backend.torrent.TorrentWrapper;
import com.github.yoep.torrent.stream.models.TorrentStreamImpl;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;
import org.springframework.util.Assert;

import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedQueue;

@Slf4j
@RequiredArgsConstructor
public class TorrentStreamServiceImpl implements TorrentStreamService {
    private final TorrentService torrentService;
    private final Queue<TorrentStream> streamCache = new ConcurrentLinkedQueue<>();

    //region TorrentStreamService

    @Override
    public TorrentStream startStream(Torrent torrent) {
        Assert.notNull(torrent, "torrent cannot be null");
        log.trace("Starting a new stream for torrent file {}", torrent.getFile());
        var torrentWrapper = TorrentWrapper.from(torrent);
        var wrapper = FxLib.INSTANCE.start_stream(PopcornFxInstance.INSTANCE.get(), torrentWrapper.getWrapperPointer());
        var filename = getFilename(torrent);
        var torrentStream = new TorrentStreamImpl(wrapper, torrentWrapper);

        log.debug("Starting stream for torrent {} at {}", filename, torrentStream.getStreamUrl());
        streamCache.add(torrentStream);

        return torrentStream;
    }

    @Override
    public void stopStream(TorrentStream torrentStream) {
        Assert.notNull(torrentStream, "torrentStream cannot be null");
        try {
            if (streamCache.contains(torrentStream)) {
                log.debug("Stopping torrentStream stream for {}", torrentStream);

                torrentStream.stopStream();
                streamCache.remove(torrentStream);
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
        this.streamCache.forEach(this::stopStream);
    }

    //endregion

    //region Functions

    private String getFilename(Torrent torrent) {
        var filePath = torrent.getFile().getAbsolutePath();

        return FilenameUtils.getName(filePath);
    }

    //endregion

}
