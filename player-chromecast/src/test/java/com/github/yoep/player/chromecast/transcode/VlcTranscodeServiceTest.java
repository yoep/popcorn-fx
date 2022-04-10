package com.github.yoep.player.chromecast.transcode;

import com.github.yoep.popcorn.backend.utils.HostUtils;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import uk.co.caprica.vlcj.factory.MediaPlayerApi;
import uk.co.caprica.vlcj.factory.MediaPlayerFactory;
import uk.co.caprica.vlcj.player.base.EventApi;
import uk.co.caprica.vlcj.player.base.MediaApi;
import uk.co.caprica.vlcj.player.base.MediaPlayer;

import java.text.MessageFormat;

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
    @InjectMocks
    private VlcTranscodeService service;

    @BeforeEach
    void setUp() {
        lenient().when(mediaPlayerFactory.mediaPlayers()).thenReturn(mediaPlayerApi);
        lenient().when(mediaPlayerApi.newMediaPlayer()).thenReturn(mediaPlayer);
        lenient().when(mediaPlayer.events()).thenReturn(eventApi);
        lenient().when(mediaPlayer.media()).thenReturn(mediaApi);
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
        var expectedUrl = MessageFormat.format("http://{0}:{1}/my-original-video.{2}", HostUtils.hostAddress(), String.valueOf(HostUtils.availablePort()),
                VlcTranscodeService.EXTENSION);
        when(mediaApi.play(url, ":sout=#transcode{vcodec=VP80,vb=300,acodec=vorb,ab=128,channels=2,samplerate=44100,threads=2}:http{mux=webm," +
                        "dst=:1001/my-original-video.webm}",
                ":sout-keep")).thenReturn(true);

        var result = service.transcode(url);

        assertEquals(expectedUrl, result);
    }
}