package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.adapter.PlayRequest;
import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
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
import java.util.Optional;

/**
 * The video service is responsible for handling the active video player and surface.
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class VideoService implements PlaybackListener {
    public static final String VIDEO_PLAYER_PROPERTY = "videoPlayer";
    private final List<VideoPlayer> videoPlayers;
    private final PopcornPlayer player;

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

    //region PlaybackListener

    @Override
    public void onPlay(PlayRequest request) {
        Assert.notNull(request, "request cannot be null");
        var url = request.getUrl();
        var videoPlayer = switchSupportedVideoPlayer(url);

        videoPlayer.play(url);
    }

    @Override
    public void onResume() {
        videoPlayer.get().resume();
    }

    @Override
    public void onPause() {
        videoPlayer.get().pause();
    }

    @Override
    public void onSeek(long time) {
        videoPlayer.get().seek(time);
    }

    @Override
    public void onVolume(int volume) {
        //TODO: implement
    }

    @Override
    public void onStop() {
        videoPlayer.get().stop();
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

        updateActiveVideoPlayer(videoPlayer);
        return videoPlayer;
    }

    private void updateActiveVideoPlayer(VideoPlayer videoPlayer) {
        var oldVideoPlayer = this.videoPlayer.get();

        this.videoPlayer.set(videoPlayer);
        player.updateActiveVideoPlayer(oldVideoPlayer, videoPlayer);
    }

    //endregion
}
