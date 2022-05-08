package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.AbstractPlaybackListener;
import com.github.yoep.player.popcorn.listeners.AbstractPlayerListener;
import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerControlsListener;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.player.PlayStreamRequest;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.screen.ScreenService;
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
public class PlayerControlsService extends AbstractListenerService<PlayerControlsListener> {
    private final PopcornPlayer player;
    private final ScreenService screenService;
    private final VideoService videoService;

    private final PlaybackListener playbackListener = createPlaybackListener();
    private final PlayerListener playerListener = createPlayerListener();
    private final TorrentListener torrentListener = createTorrentListener();

    private TorrentStream lastKnownTorrent;

    //region Methods

    public void toggleFullscreen() {
        screenService.toggleFullscreen();
    }

    public void togglePlayerPlaybackState() {
        if (player.getState() == PlayerState.PAUSED) {
            player.resume();
        } else {
            player.pause();
        }
    }

    public void onSeekChanging(Boolean isSeeking) {
        if (isSeeking) {
            player.pause();
        } else {
            player.resume();
        }
    }

    public void onVolumeChanged(double volume) {
        player.volume((int) (volume * 100));
    }

    public void seek(long time) {
        player.seek(time);
    }

    /**
     * Retrieve the initial values for the player through the listeners.
     */
    public void retrieveValues() {
        invokeListeners(e -> e.onVolumeChanged(player.getVolume()));
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        screenService.fullscreenProperty().addListener((observableValue, oldValue, newValue) -> invokeListeners(e -> e.onFullscreenStateChanged(newValue)));
        videoService.addListener(playbackListener);
        player.addListener(playerListener);
    }

    //endregion

    //region Functions

    private void onPlayRequest(PlayRequest request) {
        invokeListeners(e -> e.onSubtitleStateChanged(request.isSubtitlesEnabled()));

        Optional.ofNullable(lastKnownTorrent)
                .ifPresent(e -> e.removeListener(torrentListener));

        if (request instanceof PlayStreamRequest) {
            onStreamRequest((PlayStreamRequest) request);
        }
    }

    private void onPlayerStateChanged(PlayerState state) {
        invokeListeners(e -> e.onPlayerStateChanged(state));
    }

    private void onPlayerTimeChanged(long time) {
        invokeListeners(e -> e.onPlayerTimeChanged(time));
    }

    private void onPlayerDurationChanged(long duration) {
        invokeListeners(e -> e.onPlayerDurationChanged(duration));
    }

    private void onPlayerVolumeChanged(int volume) {
        invokeListeners(e -> e.onVolumeChanged(volume));
    }

    private void onStreamRequest(PlayStreamRequest request) {
        var torrent = request.getTorrentStream();

        torrent.addListener(torrentListener);
        this.lastKnownTorrent = torrent;
    }

    private void onStreamProgressChanged(DownloadStatus status) {
        invokeListeners(e -> e.onDownloadStatusChanged(status));
    }

    private PlayerListener createPlayerListener() {
        return new AbstractPlayerListener() {
            @Override
            public void onStateChanged(PlayerState newState) {
                onPlayerStateChanged(newState);
            }

            @Override
            public void onTimeChanged(long newTime) {
                onPlayerTimeChanged(newTime);
            }

            @Override
            public void onDurationChanged(long newDuration) {
                onPlayerDurationChanged(newDuration);
            }

            @Override
            public void onVolumeChanged(int volume) {
                onPlayerVolumeChanged(volume);
            }
        };
    }

    private PlaybackListener createPlaybackListener() {
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

    //endregion
}
