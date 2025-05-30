package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.player.PopcornPlayerException;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayerException;
import com.github.yoep.popcorn.backend.adapters.video.listeners.AbstractVideoListener;
import com.github.yoep.popcorn.backend.adapters.video.listeners.VideoListener;
import com.github.yoep.popcorn.backend.adapters.video.state.VideoState;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Player;
import com.github.yoep.popcorn.backend.services.AbstractListenerService;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.ReadOnlyObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import lombok.extern.slf4j.Slf4j;

import java.util.ArrayList;
import java.util.List;
import java.util.Objects;
import java.util.Optional;

/**
 * The video service is responsible for handling the active video player and surface.
 */
@Slf4j
public class VideoService extends AbstractListenerService<PlaybackListener> {
    public static final int HIGHEST_ORDER = Integer.MIN_VALUE;
    public static final int DEFAULT_ORDER = 0;
    public static final int LOWEST_ORDER = Integer.MAX_VALUE;
    public static final String VIDEO_PLAYER_PROPERTY = "videoPlayer";

    final List<PlaybackOrder> videoPlaybacks = new ArrayList<>();
    final ObjectProperty<VideoPlayback> videoPlayer = new SimpleObjectProperty<>(this, VIDEO_PLAYER_PROPERTY);
    final VideoListener videoListener = createVideoListener();

    //region Properties

    /**
     * Get the current active video player.
     *
     * @return Returns the active video player if one is present, else {@link Optional#empty()}.
     */
    public Optional<VideoPlayback> getVideoPlayer() {
        return Optional.ofNullable(videoPlayer.get());
    }

    /**
     * Get the active video player property.
     *
     * @return Returns the active video player property.
     */
    public ReadOnlyObjectProperty<VideoPlayback> videoPlayerProperty() {
        return videoPlayer;
    }

    //endregion

    //region Methods

    public void addVideoPlayback(VideoPlayback videoPlayback, int order) {
        Objects.requireNonNull(videoPlayback, "videoPlayback cannot be null");
        videoPlaybacks.add(new PlaybackOrder(videoPlayback, order));
        videoPlaybacks.sort(PlaybackOrder::compareTo);
    }

    public void onPlay(Player.PlayRequest request) {
        Objects.requireNonNull(request, "request cannot be null");
        var url = request.getUrl();

        try {
            videoPlayer.set(switchSupportedVideoPlayer(url));
            videoPlayer.get().play(url);

            // verify if a resume timestamp is known
            // if so, seek the given timestamp
            if (request.hasAutoResumeTimestamp()) {
                videoPlayer.get().seek(request.getAutoResumeTimestamp());
            }

            // let the listeners known that a play request was received
            invokeListeners(e -> e.onPlay(request));
        } catch (Exception ex) {
            log.error("Failed to start video playback of {}, {}", url, ex.getMessage(), ex);
            throw new PopcornPlayerException(url, ex.getMessage(), ex);
        }
    }

    public void onResume() {
        Optional.ofNullable(videoPlayer.get())
                .ifPresent(VideoPlayback::resume);
        invokeListeners(PlaybackListener::onResume);
    }

    public void onPause() {
        Optional.ofNullable(videoPlayer.get())
                .ifPresent(VideoPlayback::pause);
        invokeListeners(PlaybackListener::onPause);
    }

    public void onSeek(long time) {
        Optional.ofNullable(videoPlayer.get())
                .ifPresent(e -> e.seek(time));
        invokeListeners(e -> e.onSeek(time));
    }

    public void onVolume(int volume) {
        Optional.ofNullable(videoPlayer.get())
                .ifPresent(e -> e.volume(volume));
        invokeListeners(e -> e.onVolume(volume));
    }

    public void onStop() {
        // verify if the current status isn't already stopped
        // if so, ignore the action
        if (getVideoPlayerState() == VideoState.STOPPED) {
            return;
        }

        Optional.ofNullable(videoPlayer.get())
                .ifPresent(VideoPlayback::stop);
        invokeListeners(PlaybackListener::onStop);
    }

    public int getVolume() {
        return Optional.ofNullable(videoPlayer.get())
                .map(VideoPlayback::getVolume)
                .orElse(100);
    }

    //endregion

    void dispose() {
        log.trace("Disposing the video players");
        videoPlaybacks.forEach(e -> e.videoPlayback.dispose());
    }

    //region Functions

    /**
     * Switch the current active video player to one that supports the playback url.
     *
     * @param url The url the video player should support.
     * @return Returns the new active video player that supports the url.
     * @throws VideoPlayerException Is thrown when no video player could be found that supports the given url.
     */
    private VideoPlayback switchSupportedVideoPlayer(String url) {
        Objects.requireNonNull(url, "url cannot be null");
        var videoPlayer = videoPlaybacks.stream()
                .map(e -> e.videoPlayback)
                .filter(e -> e.supports(url))
                .findFirst()
                .orElseThrow(() -> new VideoPlayerException("No compatible video player found for " + url));
        var oldVideoPlayer = this.videoPlayer.get();

        registerListener(videoPlayer, oldVideoPlayer);
        this.videoPlayer.set(videoPlayer);
        return videoPlayer;
    }

    private void registerListener(VideoPlayback videoPlayback, VideoPlayback oldVideoPlayback) {
        // if an old video player is known
        // unregister the listener from the old video player first
        if (oldVideoPlayback != null) {
            oldVideoPlayback.removeListener(videoListener);
        }

        videoPlayback.addListener(videoListener);
    }

    private void onVideoError() {
        var videoPlayerError = getVideoPlayerError();
        log.error("Video player {} changed to state {}, {}", getVideoPlayerType(), getVideoPlayerState(), videoPlayerError.getMessage(), videoPlayerError);
    }

    private Throwable getVideoPlayerError() {
        return getVideoPlayer()
                .map(VideoPlayback::getError)
                .orElseGet(UnknownError::new);
    }

    private VideoState getVideoPlayerState() {
        return getVideoPlayer()
                .map(VideoPlayback::getVideoState)
                .orElse(VideoState.UNKNOWN);
    }

    private String getVideoPlayerType() {
        return getVideoPlayer()
                .map(e -> e.getClass().getSimpleName())
                .orElse("UNKNOWN");
    }

    private VideoListener createVideoListener() {
        return new AbstractVideoListener() {
            @Override
            public void onStateChanged(VideoState newState) {
                if (newState == VideoState.ERROR) {
                    onVideoError();
                }
            }
        };
    }

    //endregion

    record PlaybackOrder(VideoPlayback videoPlayback, int order) implements Comparable<PlaybackOrder> {
        @Override
        public int compareTo(PlaybackOrder o) {
            return Integer.compare(order, o.order);
        }
    }
}
