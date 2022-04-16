package com.github.yoep.player.chromecast.transcode;

import com.github.yoep.player.chromecast.services.TranscodeState;
import com.github.yoep.popcorn.backend.utils.HostUtils;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import uk.co.caprica.vlcj.factory.MediaPlayerApi;
import uk.co.caprica.vlcj.factory.MediaPlayerFactory;
import uk.co.caprica.vlcj.player.base.*;

import java.text.MessageFormat;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class VlcTranscodeServiceTest {
    @Mock
    private MediaPlayerFactory mediaPlayerFactory;
    @Mock
    private MediaPlayer mediaPlayer;
    @Mock
    private MediaPlayerApi mediaPlayerApi;
    @Mock
    private MediaApi mediaApi;
    @Mock
    private EventApi eventApi;
    @Mock
    private ControlsApi controlsApi;
    @InjectMocks
    private VlcTranscodeService service;

    private final AtomicReference<MediaPlayerEventListener> listenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().when(mediaPlayerFactory.mediaPlayers()).thenReturn(mediaPlayerApi);
        lenient().when(mediaPlayerApi.newMediaPlayer()).thenReturn(mediaPlayer);
        lenient().when(mediaPlayer.events()).thenReturn(eventApi);
        lenient().when(mediaPlayer.media()).thenReturn(mediaApi);
        lenient().when(mediaPlayer.controls()).thenReturn(controlsApi);
        lenient().doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, MediaPlayerEventListener.class));
            return null;
        }).when(eventApi).addMediaPlayerEventListener(isA(MediaPlayerEventListener.class));
    }

    @Test
    void testOnDestroy_whenInvoked_shouldReleaseVlcResources() {
        service.onDestroy();

        verify(mediaPlayerFactory).release();
    }

    @Test
    void testTranscode_whenTranscodingFailedToStart_shouldThrowTranscodeException() {
        var url = "http://localhost:8456/lorem.mkv";

        assertThrows(TranscodeException.class, () -> service.transcode(url));
    }

    @Test
    void testTranscode_whenUrlIsGiven_shouldReturnTheExpectedTranscodeUrl() {
        var url = "http://localhost:8754/my-original-video.mkv";
        var expectedUrl = MessageFormat.format("http://{0}:{1}/my-original-video", HostUtils.hostAddress(), String.valueOf(HostUtils.availablePort()));
        when(mediaApi.play(eq(url), isA(String.class), isA(String.class), isA(String.class), isA(String.class), isA(String.class))).thenReturn(true);

        var result = service.transcode(url);

        assertEquals(expectedUrl, result);
        assertEquals(TranscodeState.PREPARING, service.getState());
    }

    @Test
    void testStop_whenInvoked_shouldStopTheTranscodeProcess() {
        var url = "http://localhost:8754/lorem.mkv";
        when(mediaApi.play(isA(String.class), isA(String.class), isA(String.class), isA(String.class), isA(String.class), isA(String.class))).thenReturn(true);
        service.transcode(url);

        service.stop();

        assertEquals(TranscodeState.STOPPED, service.getState());
        verify(controlsApi).stop();
        verify(mediaPlayer).release();
    }

    @Test
    void testListener_whenProcessIsOpeningTheStream_shouldUpdateStateToStarting() {
        var url = "http://localhost:8754/lorem.mkv";
        when(mediaApi.play(isA(String.class), isA(String.class), isA(String.class), isA(String.class), isA(String.class), isA(String.class))).thenReturn(true);
        service.transcode(url);

        var listener = listenerHolder.get();
        listener.opening(mediaPlayer);

        assertEquals(TranscodeState.STARTING, service.getState());
    }

    @Test
    void testListener_whenProcessIsTranscodingTheStream_shouldUpdateStateToTranscoding() {
        var url = "http://localhost:8754/lorem.mkv";
        when(mediaApi.play(isA(String.class), isA(String.class), isA(String.class), isA(String.class), isA(String.class), isA(String.class))).thenReturn(true);
        service.transcode(url);

        var listener = listenerHolder.get();
        listener.playing(mediaPlayer);

        assertEquals(TranscodeState.TRANSCODING, service.getState());
    }

    @Test
    void testListener_whenProcessHasEncounteredAnError_shouldUpdateStateToError() {
        var url = "http://localhost:8754/lorem.mkv";
        when(mediaApi.play(isA(String.class), isA(String.class), isA(String.class), isA(String.class), isA(String.class), isA(String.class))).thenReturn(true);
        service.transcode(url);

        var listener = listenerHolder.get();
        listener.error(mediaPlayer);

        assertEquals(TranscodeState.ERROR, service.getState());
    }
}