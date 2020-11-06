package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.VideoPlayerException;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.ReadOnlyObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.lang.Nullable;
import org.springframework.stereotype.Service;

import javax.annotation.PreDestroy;
import java.util.List;
import java.util.Optional;

@Slf4j
@Service
@RequiredArgsConstructor
public class VideoPlayerManagerService {
    public static final String VIDEO_PLAYER_PROPERTY = "videoPlayer";

    private final List<VideoPlayer> videoPlayers;

    private final ObjectProperty<VideoPlayer> videoPlayer = new SimpleObjectProperty<>(this, VIDEO_PLAYER_PROPERTY);

    //region Properties

    /**
     * Get the current active video player of the service.
     *
     * @return Returns the active video player.
     */
    @Nullable
    public VideoPlayer getVideoPlayer() {
        return videoPlayer.get();
    }

    /**
     * Get the video player property.
     *
     * @return Returns the video player property.
     */
    public ReadOnlyObjectProperty<VideoPlayer> videoPlayerProperty() {
        return videoPlayer;
    }

    //endregion

    //region Methods

    /**
     * Get the active video player for the current playback.
     *
     * @return Returns the active video player when present, else {@link Optional#empty()}.
     */
    public synchronized Optional<VideoPlayer> getActivePlayer() {
        return Optional.ofNullable(getVideoPlayer());
    }

    synchronized void updateActiveVideoPlayer(String url) {
        var videoPlayer = videoPlayers.stream()
                .filter(e -> e.supports(url))
                .findFirst()
                .orElseThrow(() -> new VideoPlayerException("No compatible video player found for " + url));

        // check if the video player is the same
        // if so, do not update the active video player
        if (videoPlayer != this.videoPlayer)
            this.videoPlayer.set(videoPlayer);
    }

    //endregion

    //region PreDestroy

    @PreDestroy
    private void dispose() {
        videoPlayers.forEach(VideoPlayer::dispose);
    }

    //endregion
}
