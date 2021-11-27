package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.adapter.PlayRequest;
import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.VideoPlayerException;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.ReadOnlyObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import javax.annotation.PreDestroy;
import java.util.List;
import java.util.Objects;
import java.util.Optional;
import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.function.Consumer;

/**
 * The video service is responsible for handling the active video player and surface.
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class VideoService {
    public static final String VIDEO_PLAYER_PROPERTY = "videoPlayer";
    private final List<VideoPlayer> videoPlayers;

    private final Queue<PlaybackListener> playbackListeners = new ConcurrentLinkedQueue<>();
    private final ObjectProperty<VideoPlayer> videoPlayer = new SimpleObjectProperty<>(this, VIDEO_PLAYER_PROPERTY);

    //region Properties

    /**
     * Get the current active video player.
     *
     * @return Returns the active video player if one is present, else {@link Optional#empty()}.
     */
    public Optional<VideoPlayer> getVideoPlayer() {
        return Optional.ofNullable(videoPlayer.get());
    }

    /**
     * Get the active video player property.
     *
     * @return Returns the active video player property.
     */
    public ReadOnlyObjectProperty<VideoPlayer> videoPlayerProperty() {
        return videoPlayer;
    }

    //endregion

    //region Methods

    public void onPlay(PlayRequest request) {
        Assert.notNull(request, "request cannot be null");
        var url = request.getUrl();

        videoPlayer.set(switchSupportedVideoPlayer(url));
        videoPlayer.get().play(url);
        invokeListeners(e -> e.onPlay(request));
    }

    public void onResume() {
        videoPlayer.get().resume();
        invokeListeners(PlaybackListener::onResume);
    }

    public void onPause() {
        videoPlayer.get().pause();
        invokeListeners(PlaybackListener::onPause);
    }

    public void onSeek(long time) {
        videoPlayer.get().seek(time);
        invokeListeners(e -> e.onSeek(time));
    }

    public void onVolume(int volume) {
        //TODO: implement
        invokeListeners(e -> e.onVolume(volume));
    }

    public void onStop() {
        videoPlayer.get().stop();
        invokeListeners(PlaybackListener::onStop);
    }

    public void addListener(PlaybackListener listener) {
        Objects.requireNonNull(listener, "listener cannot be null");
        playbackListeners.add(listener);
    }

    //endregion

    //region PreDestroy

    @PreDestroy
    public void dispose() {
        log.trace("Disposing the video players");
        videoPlayers.forEach(VideoPlayer::dispose);
    }

    //endregion

    //region Functions

    /**
     * Switch the current active video player to one that supports the playback url.
     *
     * @param url The url the video player should support.
     * @return Returns the new active video player that supports the url.
     * @throws VideoPlayerException Is thrown when no video player could be found that supports the given url.
     */
    VideoPlayer switchSupportedVideoPlayer(String url) {
        Assert.notNull(url, "url cannot be null");
        var videoPlayer = videoPlayers.stream()
                .filter(e -> e.supports(url))
                .findFirst()
                .orElseThrow(() -> new VideoPlayerException("No compatible video player found for " + url));
        var oldVideoPlayer = this.videoPlayer.get();

        this.videoPlayer.set(videoPlayer);
        return videoPlayer;
    }

    private void invokeListeners(Consumer<PlaybackListener> action) {
        playbackListeners.forEach(listener -> {
            try {
                action.accept(listener);
            } catch (Exception ex) {
                log.error("Failed to invoke playback listener, {}", ex.getMessage(), ex);
            }
        });
    }

    //endregion
}
