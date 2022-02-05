package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.AbstractPlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerHeaderListener;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.player.PlayStreamRequest;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.AbstractTorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.listeners.TorrentListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentStream;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.util.Optional;

@Slf4j
@Service
@RequiredArgsConstructor
public class PlayerHeaderService extends AbstractListenerService<PlayerHeaderListener> {
    private final PopcornPlayer player;
    private final VideoService videoService;

    private final PlaybackListener listener = createListener();
    private final TorrentListener torrentListener = createTorrentListener();

    private TorrentStream lastKnownTorrent;

    public void stop() {
        player.stop();
    }

    @PostConstruct
    void init() {
        videoService.addListener(listener);
    }

    private void onPlayRequest(PlayRequest request) {
        invokeListeners(e -> e.onTitleChanged(request.getTitle().orElse("Unknown")));
        invokeListeners(e -> e.onQualityChanged(request.getQuality().orElse(null)));
        invokeListeners(e -> e.onStreamStateChanged(request instanceof PlayStreamRequest));

        Optional.ofNullable(lastKnownTorrent)
                .ifPresent(e -> e.removeListener(torrentListener));

        if (request instanceof PlayStreamRequest) {
            onStreamRequest((PlayStreamRequest) request);
        }
    }

    private void onStreamRequest(PlayStreamRequest request) {
        var torrent = request.getTorrentStream();

        torrent.addListener(torrentListener);
        this.lastKnownTorrent = torrent;
    }

    private void onStreamProgressChanged(DownloadStatus status) {
        invokeListeners(e -> e.onDownloadStatusChanged(status));
    }

    private PlaybackListener createListener() {
        return new AbstractPlaybackListener() {
            @Override
            public void onPlay(PlayRequest request) {
                onPlayRequest(request);
            }
        };
    }

    private TorrentListener createTorrentListener() {
        return new AbstractTorrentListener() {
            @Override
            public void onDownloadStatus(DownloadStatus status) {
                onStreamProgressChanged(status);
            }
        };
    }
}
