package com.github.yoep.video.vlc;

import com.github.yoep.video.adapter.state.PlayerState;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.stubbing.Answer;
import uk.co.caprica.vlcj.factory.MediaPlayerApi;
import uk.co.caprica.vlcj.factory.MediaPlayerFactory;
import uk.co.caprica.vlcj.player.base.EventApi;
import uk.co.caprica.vlcj.player.base.MediaPlayerEventListener;
import uk.co.caprica.vlcj.player.embedded.EmbeddedMediaPlayer;
import uk.co.caprica.vlcj.player.embedded.VideoSurfaceApi;
import uk.co.caprica.vlcj.player.embedded.videosurface.VideoSurface;

import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class VideoPlayerVlcTest {
    @Mock
    private MediaPlayerFactory mediaPlayerFactory;
    @Mock
    private EmbeddedMediaPlayer mediaPlayer;
    @Mock
    private MediaPlayerApi mediaPlayerApi;
    @Mock
    private EventApi eventApi;
    @Mock
    private VideoSurfaceApi videoSurfaceApi;

    @BeforeEach
    void setUp() {
        when(mediaPlayerFactory.mediaPlayers()).thenReturn(mediaPlayerApi);
        when(mediaPlayerApi.newEmbeddedMediaPlayer()).thenReturn(mediaPlayer);
        lenient().when(mediaPlayer.videoSurface()).thenReturn(videoSurfaceApi);
        lenient().when(mediaPlayer.events()).thenReturn(eventApi);
    }

    @Test
    void testInitializeVlcEvents_whenPlayerTimeIsChanged_shouldChangeTimeProperty() {
        var newTime = 3000L;
        var eventListenerHolder = new AtomicReference<MediaPlayerEventListener>();
        var videoPlayerVlc = createVideoPlayer(eventListenerHolder);

        var eventListener = eventListenerHolder.get();
        eventListener.timeChanged(mediaPlayer, newTime);

        assertEquals(newTime, videoPlayerVlc.getTime());
    }

    @Test
    void testInitializeVlcEvents_whenPlayerDurationIsChanged_shouldChangeDurationProperty() {
        var newDuration = 60000L;
        var eventListenerHolder = new AtomicReference<MediaPlayerEventListener>();
        var videoPlayerVlc = createVideoPlayer(eventListenerHolder);

        var eventListener = eventListenerHolder.get();
        eventListener.lengthChanged(mediaPlayer, newDuration);

        assertEquals(newDuration, videoPlayerVlc.getDuration());
    }

    @Test
    void testInitializeVlcEvents_whenPlayerIsPlaying_shouldChangeStateToPlaying() {
        var eventListenerHolder = new AtomicReference<MediaPlayerEventListener>();
        var videoPlayerVlc = createVideoPlayer(eventListenerHolder);

        var eventListener = eventListenerHolder.get();
        eventListener.playing(mediaPlayer);

        assertEquals(PlayerState.PLAYING, videoPlayerVlc.getPlayerState());
    }

    @Test
    void testInitializeVlcEvents_whenPlayerIsBufferingAndBufferIsSmallerThan100_shouldChangeStateToBuffering() {
        var eventListenerHolder = new AtomicReference<MediaPlayerEventListener>();
        var videoPlayerVlc = createVideoPlayer(eventListenerHolder);

        var eventListener = eventListenerHolder.get();
        eventListener.buffering(mediaPlayer, 10);

        assertEquals(PlayerState.BUFFERING, videoPlayerVlc.getPlayerState());
    }

    @Test
    void testInitializeVlcEvents_whenPlayerIsBufferingAndBufferIsS100_shouldChangeStateToPlaying() {
        var eventListenerHolder = new AtomicReference<MediaPlayerEventListener>();
        var videoPlayerVlc = createVideoPlayer(eventListenerHolder);

        var eventListener = eventListenerHolder.get();
        eventListener.buffering(mediaPlayer, 100);

        assertEquals(PlayerState.PLAYING, videoPlayerVlc.getPlayerState());
    }

    @Test
    void testInitializeVlcEvents_whenPlayerIsPaused_shouldChangeStateToPaused() {
        var eventListenerHolder = new AtomicReference<MediaPlayerEventListener>();
        var videoPlayerVlc = createVideoPlayer(eventListenerHolder);

        var eventListener = eventListenerHolder.get();
        eventListener.paused(mediaPlayer);

        assertEquals(PlayerState.PAUSED, videoPlayerVlc.getPlayerState());
    }

    @Test
    void testInitializeVlcEvents_whenPlayerIsStopped_shouldChangeStateToStopped() {
        var eventListenerHolder = new AtomicReference<MediaPlayerEventListener>();
        var videoPlayerVlc = createVideoPlayer(eventListenerHolder);

        var eventListener = eventListenerHolder.get();
        eventListener.stopped(mediaPlayer);

        assertEquals(PlayerState.STOPPED, videoPlayerVlc.getPlayerState());
    }

    @Test
    void testInitializeVlcEvents_whenPlayerIsFinished_shouldChangeStateToFinished() {
        var eventListenerHolder = new AtomicReference<MediaPlayerEventListener>();
        var videoPlayerVlc = createVideoPlayer(eventListenerHolder);

        var eventListener = eventListenerHolder.get();
        eventListener.finished(mediaPlayer);

        assertEquals(PlayerState.FINISHED, videoPlayerVlc.getPlayerState());
    }

    @Test
    void testInit_whenInvoked_shouldSetVideoSurfaceOfVlcPlayer() {
        var videoPlayerVlc = new VideoPlayerVlc(mediaPlayerFactory);

        videoPlayerVlc.init();

        verify(videoSurfaceApi).set(isA(VideoSurface.class));
    }

    private VideoPlayerVlc createVideoPlayer(AtomicReference<MediaPlayerEventListener> eventListenerHolder) {
        doAnswer((Answer<Void>) invocationOnMock -> {
            eventListenerHolder.set(invocationOnMock.getArgument(0));
            return null;
        }).when(eventApi).addMediaPlayerEventListener(isA(MediaPlayerEventListener.class));

        return new VideoPlayerVlc(mediaPlayerFactory);
    }
}
