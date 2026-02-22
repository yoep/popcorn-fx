package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.AbstractPlaybackListener;
import com.github.yoep.player.popcorn.listeners.AbstractPlayerListener;
import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.listeners.PlayerControlsListener;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.screen.ScreenService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ServerStream;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Stream;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import com.github.yoep.popcorn.backend.stream.IStreamServer;
import com.github.yoep.popcorn.backend.stream.StreamListener;
import com.github.yoep.popcorn.backend.stream.StreamServer;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
import java.util.Optional;

@Slf4j
public class PlayerControlsService extends AbstractListenerService<PlayerControlsListener> {
    private final PopcornPlayer player;
    private final ScreenService screenService;
    private final VideoService videoService;
    private final IStreamServer streamServer;

    private final PlaybackListener playbackListener = createPlaybackListener();
    private final PlayerListener playerListener = createPlayerListener();
    private final StreamListener streamListener = createStreamListener();

    private String filename;

    public PlayerControlsService(PopcornPlayer player, ScreenService screenService, VideoService videoService, IStreamServer streamServer) {
        Objects.requireNonNull(player, "player cannot be null");
        Objects.requireNonNull(screenService, "screenService cannot be null");
        Objects.requireNonNull(videoService, "videoService cannot be null");
        Objects.requireNonNull(streamServer, "streamServer cannot be null");
        this.player = player;
        this.screenService = screenService;
        this.videoService = videoService;
        this.streamServer = streamServer;
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
        if (player.getState() == Player.State.PAUSED) {
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

    private void onPlayRequest(Player.PlayRequest request) {
        invokeListeners(e -> e.onSubtitleStateChanged(request.getSubtitle().getEnabled()));
        Optional.ofNullable(this.filename)
                .ifPresent(handle -> streamServer.removeListener(handle, streamListener));
        Optional.ofNullable(request.getStream())
                .filter(e -> request.hasStream())
                .map(ServerStream::getFilename)
                .ifPresent(filename -> {
                    this.filename = filename;
                    streamServer.addListener(filename, streamListener);
                });
    }

    private void onPlayerStateChanged(Player.State state) {
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


    private void onStreamProgressChanged(Stream.StreamStats stats) {
        invokeListeners(e -> e.onDownloadStatusChanged(new DownloadStatus() {
            @Override
            public float progress() {
                return stats.getProgress();
            }

            @Override
            public long connections() {
                return stats.getConnections();
            }

            @Override
            public int downloadSpeed() {
                return stats.getDownloadSpeed();
            }

            @Override
            public int uploadSpeed() {
                return stats.getUploadSpeed();
            }

            @Override
            public long downloaded() {
                return stats.getDownloaded();
            }

            @Override
            public long totalSize() {
                return stats.getTotalSize();
            }
        }));
    }

    private PlayerListener createPlayerListener() {
        return new AbstractPlayerListener() {
            @Override
            public void onStateChanged(Player.State newState) {
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
            public void onPlay(Player.PlayRequest request) {
                onPlayRequest(request);
            }
        };
    }

    private StreamListener createStreamListener() {
        return new StreamListener() {
            @Override
            public void onStateChanged(Stream.StreamState state) {
                // no-op
            }

            @Override
            public void onStatsChanged(Stream.StreamStats stats) {
                onStreamProgressChanged(stats);
            }
        };
    }

    //endregion
}
