package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.AbstractPlaybackListener;
import com.github.yoep.player.popcorn.listeners.AbstractPlayerListener;
import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerControlsListener;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.adapters.screen.ScreenService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentStreamListener;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.adapters.torrent.state.TorrentStreamState;
import com.github.yoep.popcorn.backend.lib.Handle;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import lombok.extern.slf4j.Slf4j;

import java.util.Optional;

@Slf4j
public class PlayerControlsService extends AbstractListenerService<PlayerControlsListener> {
    private final PopcornPlayer player;
    private final ScreenService screenService;
    private final VideoService videoService;
    private final TorrentService torrentService;

    private final PlaybackListener playbackListener = createPlaybackListener();
    private final PlayerListener playerListener = createPlayerListener();
    private final TorrentStreamListener torrentStreamListener = createStreamListener();

    private Handle lastKnownCallbackHandle;

    public PlayerControlsService(PopcornPlayer player, ScreenService screenService, VideoService videoService, TorrentService torrentService) {
        this.player = player;
        this.screenService = screenService;
        this.videoService = videoService;
        this.torrentService = torrentService;
        init();
    }

    //region Methods

    public long getTime() {
        return player.getTime();
    }

    public void toggleFullscreen() {
        screenService.toggleFullscreen();
    }

    public void pause() {
        player.pause();
    }

    public void resume() {
        player.resume();
    }

    public void togglePlayerPlaybackState() {
        if (player.getState() == PlayerState.PAUSED) {
            resume();
        } else {
            pause();
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

    private void init() {
        screenService.fullscreenProperty().addListener((observableValue, oldValue, newValue) -> invokeListeners(e -> e.onFullscreenStateChanged(newValue)));
        videoService.addListener(playbackListener);
        player.addListener(playerListener);
    }

    //endregion

    //region Functions

    private void onPlayRequest(PlayRequest request) {
        invokeListeners(e -> e.onSubtitleStateChanged(request.isSubtitlesEnabled()));

        Optional.ofNullable(lastKnownCallbackHandle)
                .ifPresent(torrentService::removeListener);
        lastKnownCallbackHandle = request.getStreamHandle()
                .map(e -> torrentService.addListener(e, torrentStreamListener))
                .orElse(null);
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

    private TorrentStreamListener createStreamListener() {
        return new TorrentStreamListener() {
            @Override
            public void onStateChanged(TorrentStreamState newState) {
                // no-op
            }

            @Override
            public void onDownloadStatus(DownloadStatus downloadStatus) {
                onStreamProgressChanged(downloadStatus);
            }
        };
    }

    //endregion
}
