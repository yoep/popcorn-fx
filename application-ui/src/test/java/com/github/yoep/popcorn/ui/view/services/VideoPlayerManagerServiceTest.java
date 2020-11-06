package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.adapter.VideoPlayerException;
import com.github.yoep.video.adapter.VideoPlayerNotInitializedException;
import com.github.yoep.video.adapter.state.PlayerState;
import javafx.beans.property.ReadOnlyLongProperty;
import javafx.beans.property.ReadOnlyObjectProperty;
import javafx.scene.Node;
import org.junit.jupiter.api.Test;

import java.io.File;
import java.util.ArrayList;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.*;

class VideoPlayerManagerServiceTest {
    @Test
    void testUpdateActiveVideoPlayer_whenNoCompatibleVideoPlayerIsFound_shouldThrowVideoPlayerException() {
        var url = "lorem ipsum dolor";
        var videoPlayers = new ArrayList<VideoPlayer>();
        var videoPlayerManagerService = new VideoPlayerManagerService(videoPlayers);

        assertThrows(VideoPlayerException.class, () -> videoPlayerManagerService.updateActiveVideoPlayer(url), "No compatible video player found for " + url);
    }

    @Test
    void testUpdateActiveVideoPlayer_whenInvoked_shouldSelectTheCorrectPlayer() {
        var url = "https://www.vimeo.com/lorem";
        VideoPlayer youtubePlayer = new YoutubePlayer();
        VideoPlayer vimeoPlayer = new VimeoPlayer();
        var videoPlayers = asList(youtubePlayer, vimeoPlayer);
        var videoPlayerManagerService = new VideoPlayerManagerService(videoPlayers);

        videoPlayerManagerService.updateActiveVideoPlayer(url);

        var result = videoPlayerManagerService.getVideoPlayer();
        assertEquals(vimeoPlayer, result);
    }

    @Test
    void testGetActivePlayer_whenNoPlayerIsActive_shouldReturnEmpty() {
        var videoPlayers = new ArrayList<VideoPlayer>();
        var videoPlayerManagerService = new VideoPlayerManagerService(videoPlayers);

        var result = videoPlayerManagerService.getActivePlayer();

        assertTrue(result.isEmpty());
    }

    private static class YoutubePlayer extends AbstractPlayer {
        @Override
        public boolean supports(String url) {
            return url.contains("youtube");
        }
    }

    private static class VimeoPlayer extends AbstractPlayer {
        @Override
        public boolean supports(String url) {
            return url.contains("vimeo");
        }
    }

    private static abstract class AbstractPlayer implements VideoPlayer {
        @Override
        public PlayerState getPlayerState() {
            return null;
        }

        @Override
        public ReadOnlyObjectProperty<PlayerState> playerStateProperty() {
            return null;
        }

        @Override
        public long getTime() {
            return 0;
        }

        @Override
        public ReadOnlyLongProperty timeProperty() {
            return null;
        }

        @Override
        public long getDuration() {
            return 0;
        }

        @Override
        public ReadOnlyLongProperty durationProperty() {
            return null;
        }

        @Override
        public boolean supports(String url) {
            return false;
        }

        @Override
        public boolean isInitialized() {
            return false;
        }

        @Override
        public Throwable getError() {
            return null;
        }

        @Override
        public Node getVideoSurface() {
            return null;
        }

        @Override
        public void dispose() {

        }

        @Override
        public void play(String url) throws VideoPlayerNotInitializedException {

        }

        @Override
        public void pause() throws VideoPlayerNotInitializedException {

        }

        @Override
        public void resume() throws VideoPlayerNotInitializedException {

        }

        @Override
        public void seek(long time) throws VideoPlayerNotInitializedException {

        }

        @Override
        public void stop() {

        }

        @Override
        public boolean supportsNativeSubtitleFile() {
            return false;
        }

        @Override
        public void subtitleFile(File file) {

        }

        @Override
        public void subtitleDelay(long delay) {

        }
    }
}
