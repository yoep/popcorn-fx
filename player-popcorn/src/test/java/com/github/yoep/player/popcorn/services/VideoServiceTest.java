package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayer;
import com.github.yoep.popcorn.backend.player.model.SimplePlayRequest;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class VideoServiceTest {
    @Mock
    private VideoPlayer videoPlayer1;
    @Mock
    private VideoPlayer videoPlayer2;
    @Mock
    private PlaybackListener listener;

    private VideoService service;

    @BeforeEach
    void setUp() {
        service = new VideoService(asList(videoPlayer1, videoPlayer2));
    }

    @Test
    void testOnPlay_whenPlaybackIsDifferent_shouldSwitchVideoPlayers() {
        var url1 = "lorem.mp4";
        var url2 = "https://ipsum.com/test.mp4";
        var request1 = SimplePlayRequest.builder()
                .url(url1)
                .build();
        var request2 = SimplePlayRequest.builder()
                .url(url2)
                .build();
        when(videoPlayer1.supports(url1)).thenReturn(true);
        when(videoPlayer2.supports(url2)).thenReturn(true);
        service.onPlay(request1);

        service.onPlay(request2);
        var result = service.getVideoPlayer();

        assertTrue(result.isPresent(), "Expected a video player to be active");
        assertEquals(videoPlayer2, result.get());
    }

    @Test
    void testOnPlay_whenInvoked_shouldInvokeListenersWithRequest() {
        var url = "my-video-url.mp4";
        var request = SimplePlayRequest.builder()
                .url(url)
                .build();
        when(videoPlayer1.supports(url)).thenReturn(true);
        service.addListener(listener);

        service.onPlay(request);

        verify(listener).onPlay(request);
    }

    @Test
    void testOnPlay_whenAutoResumeTimestampIsKnown_shouldSeekTheTimestamp() {
        var url = "continue-video.mp4";
        var timestamp = 20000L;
        var request = SimplePlayRequest.builder()
                .url(url)
                .autoResumeTimestamp(timestamp)
                .build();
        when(videoPlayer2.supports(url)).thenReturn(true);

        service.onPlay(request);

        verify(videoPlayer2).seek(timestamp);
    }

    @Test
    void testDispose_whenInvokeD_shouldDisposeAllVideoPlayers() {
        service.dispose();

        verify(videoPlayer1).dispose();
        verify(videoPlayer2).dispose();
    }
}