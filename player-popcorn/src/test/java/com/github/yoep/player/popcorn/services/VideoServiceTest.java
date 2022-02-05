package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PlaybackListener;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.adapters.video.listeners.VideoListener;
import com.github.yoep.popcorn.backend.adapters.video.state.VideoState;
import com.github.yoep.popcorn.backend.player.model.SimplePlayRequest;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.atomic.AtomicReference;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class VideoServiceTest {
    @Mock
    private VideoPlayback videoPlayback1;
    @Mock
    private VideoPlayback videoPlayback2;
    @Mock
    private PlaybackListener listener;

    private VideoService service;

    private final AtomicReference<VideoListener> videoListener = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocation -> {
            videoListener.set(invocation.getArgument(0, VideoListener.class));
            return null;
        }).when(videoPlayback1).addListener(isA(VideoListener.class));
        service = new VideoService(asList(videoPlayback1, videoPlayback2));
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
        when(videoPlayback1.supports(url1)).thenReturn(true);
        when(videoPlayback2.supports(url2)).thenReturn(true);
        service.onPlay(request1);

        service.onPlay(request2);
        var result = service.getVideoPlayer();

        assertTrue(result.isPresent(), "Expected a video player to be active");
        assertEquals(videoPlayback2, result.get());
    }

    @Test
    void testOnPlay_whenInvoked_shouldInvokeListenersWithRequest() {
        var url = "my-video-url.mp4";
        var request = SimplePlayRequest.builder()
                .url(url)
                .build();
        when(videoPlayback1.supports(url)).thenReturn(true);
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
        when(videoPlayback2.supports(url)).thenReturn(true);

        service.onPlay(request);

        verify(videoPlayback2).seek(timestamp);
    }

    @Test
    void testOnResume_whenInvoked_shouldInvokeResumeOnTheVideoPlayerAndListeners() {
        var url = "resume-video.mp4";
        var request = SimplePlayRequest.builder()
                .url(url)
                .build();
        when(videoPlayback1.supports(url)).thenReturn(true);
        service.addListener(listener);
        service.onPlay(request);

        service.onResume();

        verify(videoPlayback1).resume();
        verify(listener).onResume();
    }

    @Test
    void testOnPause_whenInvoked_shouldInvokePauseOnTheVideoPlayerAndListeners() {
        var url = "pause-video.mp4";
        var request = SimplePlayRequest.builder()
                .url(url)
                .build();
        when(videoPlayback1.supports(url)).thenReturn(true);
        service.addListener(listener);
        service.onPlay(request);

        service.onPause();

        verify(videoPlayback1).pause();
        verify(listener).onPause();
    }

    @Test
    void testOnSeek_whenInvoked_shouldInvokeSeekOnTheVideoPlayerAndListeners() {
        var url = "seek-time-video.mp4";
        var time = 17500;
        var request = SimplePlayRequest.builder()
                .url(url)
                .build();
        when(videoPlayback1.supports(url)).thenReturn(true);
        service.addListener(listener);
        service.onPlay(request);

        service.onSeek(time);

        verify(videoPlayback1).seek(time);
        verify(listener).onSeek(time);
    }

    @Test
    void testOnVolume_whenInvoked_shouldInvokeVolumeOnTheVideoPlayerAndListeners() {
        var url = "volume-time-video.mp4";
        var volume = 90;
        var request = SimplePlayRequest.builder()
                .url(url)
                .build();
        when(videoPlayback1.supports(url)).thenReturn(true);
        service.addListener(listener);
        service.onPlay(request);

        service.onVolume(volume);

        verify(listener).onVolume(volume);
    }

    @Test
    void testOnStop_whenInvoked_shouldInvokeStopOnTheVideoPlayerAndListeners() {
        var url = "stop-video.mp4";
        var request = SimplePlayRequest.builder()
                .url(url)
                .build();
        when(videoPlayback1.supports(url)).thenReturn(true);
        service.addListener(listener);
        service.onPlay(request);

        service.onStop();

        verify(videoPlayback1).stop();
        verify(listener).onStop();
    }

    @Test
    void testDispose_whenInvokeD_shouldDisposeAllVideoPlayers() {
        service.dispose();

        verify(videoPlayback1).dispose();
        verify(videoPlayback2).dispose();
    }

    @Test
    void testVideoListener_whenOnStateChangedIsErrorState_shouldRetrieveTheVideoError() {
        var url = "my-video.mp4";
        var request = SimplePlayRequest.builder()
                .url(url)
                .build();
        when(videoPlayback1.supports(url)).thenReturn(true);
        when(videoPlayback1.getError()).thenReturn(new RuntimeException("My video player error"));
        service.addListener(listener);
        service.onPlay(request);

        videoListener.get().onStateChanged(VideoState.ERROR);

        verify(videoPlayback1).getError();
    }
}