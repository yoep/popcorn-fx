package com.github.yoep.player.popcorn;

import com.github.yoep.player.adapter.EmbeddablePlayer;
import com.github.yoep.player.adapter.LayoutMode;
import com.github.yoep.player.adapter.PlayRequest;
import com.github.yoep.player.adapter.listeners.PlayerListener;
import com.github.yoep.player.adapter.state.PlayerState;
import com.github.yoep.player.popcorn.services.VideoService;
import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.listeners.VideoListener;
import com.github.yoep.video.adapter.state.VideoState;
import javafx.scene.Node;
import lombok.EqualsAndHashCode;
import lombok.ToString;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.ClassPathResource;
import org.springframework.core.io.Resource;
import org.springframework.util.Assert;

import java.util.Collection;
import java.util.Optional;
import java.util.concurrent.ConcurrentLinkedQueue;

@Slf4j
@ToString(exclude = {"videoListener", "listeners", "embeddablePlayer"})
@EqualsAndHashCode(exclude = {"videoListener", "listeners", "embeddablePlayer"})
public class PopcornPlayer implements EmbeddablePlayer {
    public static final String PLAYER_ID = "internalPlayer";
    public static final String PLAYER_NAME = "Popcorn Time";

    static final Resource GRAPHIC_RESOURCE = new ClassPathResource("/internal-popcorn-icon.png");

    private final Collection<PlayerListener> listeners = new ConcurrentLinkedQueue<>();
    private final VideoListener videoListener = createVideoListener();
    private final VideoService videoService;
    private final Node embeddablePlayer;

    private PlayerState playerState;
    private Long time;
    private Long duration;

    public PopcornPlayer(VideoService videoService, Node embeddablePlayer) {
        Assert.notNull(videoService, "videoService cannot be null");
        Assert.notNull(embeddablePlayer, "embeddablePlayer cannot be null");
        this.videoService = videoService;
        this.embeddablePlayer = embeddablePlayer;

        init();
    }

    //region EmbeddablePlayer

    @Override
    public String getId() {
        return PLAYER_ID;
    }

    @Override
    public String getName() {
        return PLAYER_NAME;
    }

    @Override
    public Optional<Resource> getGraphicResource() {
        return Optional.of(GRAPHIC_RESOURCE);
    }

    @Override
    public PlayerState getState() {
        return null;
    }

    @Override
    public boolean isEmbeddedPlaybackSupported() {
        return true;
    }

    @Override
    public void dispose() {
        log.debug("Disposing the popcorn player");
        videoService.dispose();
    }

    @Override
    public void addListener(PlayerListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        listeners.add(listener);
    }

    @Override
    public void removeListener(PlayerListener listener) {
        listeners.remove(listener);
    }

    @Override
    public void play(PlayRequest request) {
        Assert.notNull(request, "request cannot be null");
        var url = request.getUrl();
        var videoPlayer = videoService.switchSupportedVideoPlayer(url);

        videoPlayer.play(url);
    }

    @Override
    public void resume() {
        videoService.getVideoPlayer().ifPresentOrElse(
                VideoPlayer::resume,
                () -> log.warn("Unable to resume video playback, no active video player")
        );
    }

    @Override
    public void pause() {
        videoService.getVideoPlayer().ifPresentOrElse(
                VideoPlayer::pause,
                () -> log.warn("Unable to pause video playback, no active video player")
        );
    }

    @Override
    public void stop() {
        log.trace("Stopping video playback");
        videoService.getVideoPlayer().ifPresentOrElse(
                VideoPlayer::stop,
                () -> log.warn("Unable to stop video playback, no active video player")
        );
    }

    @Override
    public void seek(long time) {
        log.trace("Updating video playback time to {}", time);
        videoService.getVideoPlayer().ifPresentOrElse(
                videoPlayer -> videoPlayer.seek(time),
                () -> log.warn("Unable to seek video playback, no active video player")
        );
    }

    @Override
    public void volume(int volume) {
        log.trace("Updating video playback volume to {}", volume);
        videoService.getVideoPlayer().ifPresentOrElse(
                e -> {
                },
                () -> log.warn("Unable to volume video playback, no active video player")
        );
    }

    @Override
    public Node getEmbeddedPlayer() {
        return embeddablePlayer;
    }

    @Override
    public void setLayoutMode(LayoutMode mode) {
        //TODO: implement
    }

    //endregion

    //region Functions

    private void init() {
        videoService.videoPlayerProperty().addListener((observable, oldValue, newValue) -> switchActiveVideoPlayer(oldValue, newValue));
    }

    private void setPlayerState(PlayerState playerState) {
        this.playerState = playerState;
        listeners.forEach(e -> e.onStateChanged(playerState));
    }

    private void setTime(Long time) {
        this.time = time;
        listeners.forEach(e -> e.onTimeChanged(time));
    }

    private void setDuration(Long duration) {
        this.duration = duration;
        listeners.forEach(e -> e.onDurationChanged(duration));
    }

    private void onVideoStateChanged(VideoState newState) {
        switch (newState) {
            case BUFFERING:
                setPlayerState(PlayerState.BUFFERING);
                break;
            case PLAYING:
                setPlayerState(PlayerState.PLAYING);
                break;
            case PAUSED:
                setPlayerState(PlayerState.PAUSED);
                break;
            case STOPPED:
            case FINISHED:
                setPlayerState(PlayerState.STOPPED);
                break;
            case ERROR:
                setPlayerState(PlayerState.ERROR);
                break;
            case UNKNOWN:
            default:
                setPlayerState(PlayerState.UNKNOWN);
                break;
        }
    }

    private void switchActiveVideoPlayer(VideoPlayer oldPlayer, VideoPlayer newPlayer) {
        Optional.ofNullable(oldPlayer)
                .ifPresent(e -> e.removeListener(videoListener));
        Optional.ofNullable(newPlayer)
                .ifPresent(e -> e.addListener(videoListener));
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
        };
    }

    //endregion
}
