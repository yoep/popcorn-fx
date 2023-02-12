package com.github.yoep.popcorn.backend.torrent;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentException;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.Torrent;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedQueue;

@Slf4j
@RequiredArgsConstructor
public class TorrentStreamServiceImpl implements TorrentStreamService {
    private final FxLib lib;
    private final PopcornFxInstance instance;
    private final TorrentService torrentService;
    private final Queue<TorrentStream> streamCache = new ConcurrentLinkedQueue<>();

    //region TorrentStreamService

    @Override
    public TorrentStream startStream(Torrent torrent) {
        Objects.requireNonNull(torrent, "torrent cannot be null");
        synchronized (streamCache) {
            log.trace("Starting a new stream for torrent file {}", torrent.getFile());
            var torrentWrapper = TorrentWrapper.from(torrent);
            var torrentStream = lib.start_stream(instance.get(), torrentWrapper.getWrapperPointer());

            log.debug("Starting stream for torrent {} at {}", torrent.getFile(), torrentStream.getStreamUrl());
            torrentStream.updateTorrent(torrentWrapper);
            streamCache.add(torrentStream);

            return torrentStream;
        }
    }

    @Override
    public void stopStream(TorrentStream torrentStream) {
        Objects.requireNonNull(torrentStream, "torrentStream cannot be null");
        synchronized (streamCache) {
            try {
                if (streamCache.contains(torrentStream)) {
                    log.debug("Stopping torrentStream stream for {}", torrentStream);
                    if (torrentStream.getTorrent() instanceof TorrentWrapper wrapper) {
                        torrentService.remove(wrapper.getTorrent());
                    }
                    if (torrentStream instanceof TorrentStreamWrapper wrapper) {
                        lib.stop_stream(instance.get(), wrapper);
                        lib.dispose_torrent_stream(wrapper);
                    }

                    torrentStream.stopStream();
                    streamCache.remove(torrentStream);
                } else {
                    log.warn("Unable to stop torrentStream stream, torrentStream is unknown ({})", torrentStream);
                }
            } catch (TorrentException ex) {
                log.error("Failed to stop torrent stream, {}", ex.getMessage(), ex);
            }
        }
    }

    @Override
    public void stopAllStreams() {
        this.streamCache.forEach(this::stopStream);
    }

    //endregion

}
