package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.controllers.sections.PopcornPlayerSectionController;
import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.VideoPlayerException;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.ReadOnlyObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import java.util.List;
import java.util.Optional;

/**
 * The video service is responsible for handling the active video player and surface.
 */
@Slf4j
@Service
@RequiredArgsConstructor
public class VideoService {
    public static final String VIDEO_PLAYER_PROPERTY = "videoPlayer";
    private final List<VideoPlayer> videoPlayers;
    private final PopcornPlayerSectionController playerSectionController;

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

    /**
     * Switch the current active video player to one that supports the playback url.
     *
     * @param url The url the video player should support.
     * @return Returns the new active video player that supports the url.
     * @throws VideoPlayerException Is thrown when no video player could be found that supports the given url.
     */
    public VideoPlayer switchSupportedVideoPlayer(String url) {
        Assert.notNull(url, "url cannot be null");
        var videoPlayer = videoPlayers.stream()
                .filter(e -> e.supports(url))
                .findFirst()
                .orElseThrow(() -> new VideoPlayerException("No compatible video player found for " + url));

        updateActiveVideoPlayer(videoPlayer);
        return videoPlayer;
    }

    /**
     * Dispose all the {@link VideoPlayer}'s managed by the video service.
     */
    public void dispose() {
        log.trace("Disposing the video players");
        videoPlayers.forEach(VideoPlayer::dispose);
    }

    //endregion

    //region Functions

    private void updateActiveVideoPlayer(VideoPlayer videoPlayer) {
        this.videoPlayer.set(videoPlayer);
        playerSectionController.setVideoView(videoPlayer.getVideoSurface());
    }

    //endregion
}
