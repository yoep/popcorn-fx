package com.github.yoep.player.popcorn.player;

import com.github.yoep.player.popcorn.services.VideoService;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.adapters.video.listeners.VideoListener;
import com.github.yoep.popcorn.backend.adapters.video.state.VideoState;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import javafx.scene.Node;
import lombok.Getter;
import lombok.extern.slf4j.Slf4j;

import java.io.InputStream;
import java.util.Collection;
import java.util.Objects;
import java.util.Optional;
import java.util.concurrent.ConcurrentLinkedQueue;

@Slf4j
public class PopcornPlayer implements com.github.yoep.popcorn.backend.adapters.player.Player {
    public static final String PLAYER_ID = "internalPlayer";
    public static final String PLAYER_NAME = "Popcorn Time";

    static final String GRAPHIC_RESOURCE = "/internal-popcorn-icon.png";

    private final Collection<PlayerListener> listeners = new ConcurrentLinkedQueue<>();
    private final VideoListener videoListener = createVideoListener();
    private final VideoService videoService;

    private Player.State playerState = Player.State.UNKNOWN;
    @Getter
    private Long time;

    public PopcornPlayer(VideoService videoService) {
        Objects.requireNonNull(videoService, "videoService cannot be null");
        this.videoService = videoService;
        init();
    }

    //region Player

    @Override
    public String getId() {
        return PLAYER_ID;
    }

    @Override
    public String getName() {
        return PLAYER_NAME;
    }

    @Override
    public String getDescription() {
        return "Popcorn Time is the default embedded player of the application. " +
                "It allows the video playback within the application itself.";
    }

    @Override
    public Optional<InputStream> getGraphicResource() {
        try {
            return Optional.ofNullable(PopcornPlayer.class.getResourceAsStream(GRAPHIC_RESOURCE));
        } catch (Exception e) {
            log.error("Failed to load graphic resource", e);
            return Optional.empty();
        }
    }

    @Override
    public com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player.State getState() {
        return playerState;
    }

    @Override
    public boolean isEmbeddedPlaybackSupported() {
        return false;
    }

    @Override
    public Optional<Node> getEmbeddedPlayer() {
        return Optional.empty();
    }

    @Override
    public void dispose() {
        log.debug("Disposing the popcorn player");
    }

    @Override
    public void addListener(PlayerListener listener) {
        Objects.requireNonNull(listener, "listener cannot be null");
        listeners.add(listener);
    }

    @Override
    public void removeListener(PlayerListener listener) {
        listeners.remove(listener);
    }

    @Override
    public void play(Player.PlayRequest request) {
        Objects.requireNonNull(request, "request cannot be null");
        videoService.onPlay(request);
    }

    @Override
    public void resume() {
        videoService.onResume();
    }

    @Override
    public void pause() {
        videoService.onPause();
    }

    @Override
    public void stop() {
        log.trace("Stopping video playback");
        videoService.onStop();
    }

    @Override
    public void seek(long time) {
        log.trace("Updating video playback time to {}", time);
        videoService.onSeek(time);
    }

    @Override
    public void volume(int volume) {
        log.trace("Updating video playback volume to {}", volume);
        videoService.onVolume(volume);
    }

    @Override
    public int getVolume() {
        return videoService.getVolume();
    }

    //endregion

    //region Init

    private void init() {
        videoService.videoPlayerProperty().addListener((observable, oldValue, newValue) -> {
            onVideoPlayerChanged(oldValue, newValue);
        });
        setPlayerState(Player.State.READY);
    }

    //endregion

    //region Functions

    private void setPlayerState(Player.State playerState) {
        this.playerState = playerState;
        listeners.forEach(e -> e.onStateChanged(playerState));
    }

    private void setTime(Long time) {
        this.time = time;
        listeners.forEach(e -> e.onTimeChanged(time));
    }

    private void setDuration(Long duration) {
        listeners.forEach(e -> e.onDurationChanged(duration));
    }

    private void onVideoPlayerChanged(VideoPlayback oldValue, VideoPlayback newValue) {
        Optional.ofNullable(oldValue)
                .ifPresent(e -> e.removeListener(videoListener));
        Optional.ofNullable(newValue)
                .ifPresent(e -> e.addListener(videoListener));
    }

    private void onVideoStateChanged(VideoState newState) {
        switch (newState) {
            case BUFFERING:
                setPlayerState(Player.State.BUFFERING);
                break;
            case PLAYING:
                setPlayerState(Player.State.PLAYING);
                break;
            case PAUSED:
                setPlayerState(Player.State.PAUSED);
                break;
            case STOPPED:
            case FINISHED:
                setPlayerState(Player.State.STOPPED);
                break;
            case ERROR:
                setPlayerState(Player.State.ERROR);
                break;
            case UNKNOWN:
            default:
                setPlayerState(Player.State.UNKNOWN);
                break;
        }
    }

    private void onVolumeChanged(int volume) {
        listeners.forEach(e -> e.onVolumeChanged(volume));
    }

    private VideoListener createVideoListener() {
        return new VideoListener() {
            @Override
            public void onDurationChanged(long newDuration) {
                setDuration(newDuration);
            }

            @Override
            public void onTimeChanged(long newTime) {
                setTime(newTime);
            }

            @Override
            public void onStateChanged(VideoState newState) {
                onVideoStateChanged(newState);
            }

            @Override
            public void onVolumeChanged(int volume) {
                PopcornPlayer.this.onVolumeChanged(volume);
            }
        };
    }

    //endregion
}
