package com.github.yoep.player.chromecast;

import com.github.yoep.player.chromecast.api.v2.Load;
import com.github.yoep.player.chromecast.api.v2.Media;
import com.github.yoep.player.chromecast.services.ChromecastService;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.adapters.player.listeners.PlayerListener;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.player.model.SimplePlayRequest;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import su.litvak.chromecast.api.v2.Application;
import su.litvak.chromecast.api.v2.ChromeCast;
import su.litvak.chromecast.api.v2.MediaStatus;
import su.litvak.chromecast.api.v2.TestMediaStatus;

import java.io.IOException;
import java.util.Collections;
import java.util.concurrent.TimeoutException;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class ChromecastPlayerTest {
    @Mock
    private ChromeCast chromeCast;
    @Mock
    private ChromecastService service;
    @InjectMocks
    private ChromecastPlayer player;

    @Test
    void testGetId_whenInvoked_shouldReturnTheChromecastName() {
        var name = "my-chromecast-name";
        when(chromeCast.getName()).thenReturn(name);

        var result = player.getId();

        assertEquals(name, result);
    }

    @Test
    void testGetName_whenInvoked_shouldReturnChromecastTitle() {
        var title = "my-chromecast-title";
        when(chromeCast.getTitle()).thenReturn(title);

        var result = player.getName();

        assertEquals(title, result);
    }

    @Test
    void testDescription_whenInvoked_shouldReturnTheExpectedDescription() {
        var result = player.getDescription();

        assertEquals(ChromecastPlayer.DESCRIPTION, result);
    }

    @Test
    void testGetGraphicsNode_whenInvoked_shouldReturnTheExpectedNode() {
        var result = player.getGraphicResource();

        assertTrue(result.isPresent(), "Expected a graphics node to be present");
        assertEquals(ChromecastPlayer.GRAPHIC_RESOURCE, result.get());
    }

    @Test
    void testIsEmbeddedPlaybackSupported_whenInvoked_shouldReturnFalse() {
        var result = player.isEmbeddedPlaybackSupported();

        assertFalse(result, "Expected the chromecast player to not support embedded playback");
    }

    @Test
    void testResume_whenInvoked_shouldResumeTheChromecastPlayer() throws IOException {
        player.resume();

        verify(chromeCast).play();
    }

    @Test
    void testPause_whenInvoked_shouldPauseTheChromecastPlayer() throws IOException {
        player.pause();

        verify(chromeCast).pause();
    }

    @Test
    void testStop_whenInvoked_shouldStopTheChromecastPlayer() throws IOException {
        player.stop();

        verify(chromeCast, timeout(500)).stopApp();
    }

    @Test
    void testSeek_whenTimeIsGiven_shouldSeekChromecastPlayer() throws IOException {
        var time = 845500L;
        var expectedResult = 845.5;
        when(service.toChromecastTime(time)).thenReturn(expectedResult);

        player.seek(time);

        verify(chromeCast).seek(expectedResult);
    }

    @Test
    void testVolume_whenVolumeIsGiven_shouldChangeChromecastVolume() throws IOException {
        var volume = 50;

        player.volume(volume);

        verify(chromeCast).setVolume(0.5f);
    }

    @Test
    void testPlay_whenRequestIsGiven_shouldStartPlayback() throws IOException {
        var request = SimplePlayRequest.builder()
                .url("http://localhost/my-video.mp4")
                .title("lorem ipsum")
                .build();
        var sessionId = "mySessionId";
        var application = new Application("1", "", "", sessionId, "", false, false, "", Collections.emptyList());
        var loadRequest = Load.builder()
                .sessionId(sessionId)
                .media(Media.builder()
                        .duration(15000.00)
                        .build())
                .build();
        when(chromeCast.launchApp(ChromecastPlayer.MEDIA_RECEIVER_APP_ID)).thenReturn(application);
        when(service.toLoadRequest(sessionId, request)).thenReturn(loadRequest);

        player.play(request);

        verify(chromeCast, timeout(500)).send(ChromecastPlayer.MEDIA_NAMESPACE, loadRequest);
    }


    @Test
    void testPlay_whenSendResultsInAnException_shouldUpdateStateToError() throws IOException, TimeoutException {
        var request = SimplePlayRequest.builder()
                .url("http://localhost/my-video.mp4")
                .title("lorem ipsum")
                .build();
        var sessionId = "mySessionId";
        var application = new Application("1", "", "", sessionId, "", false, false, "", Collections.emptyList());
        var loadRequest = Load.builder()
                .sessionId(sessionId)
                .media(Media.builder()
                        .duration(15000.00)
                        .build())
                .build();
        var listener = mock(PlayerListener.class);
        when(chromeCast.launchApp(ChromecastPlayer.MEDIA_RECEIVER_APP_ID)).thenReturn(application);
        when(service.toLoadRequest(sessionId, request)).thenReturn(loadRequest);
        doThrow(new IOException("A Chromecast error occurred")).when(chromeCast).send(isA(String.class), isA(Load.class));

        player.addListener(listener);
        player.play(request);

        verify(chromeCast, timeout(200)).send(isA(String.class), isA(Load.class));
        verify(listener, timeout(200)).onStateChanged(PlayerState.ERROR);
        assertEquals(PlayerState.ERROR, player.getState());
    }

    @Test
    void testMediaStatusChanged_whenDurationIsMaxValueDueToLiveStream_shouldUseOriginalVideoLengthAsDuration() throws IOException {
        var url = "http://localhost/my-video.mp4";
        var originalVideoDuration = Double.valueOf(35000.00);
        var request = SimplePlayRequest.builder()
                .url(url)
                .title("lorem ipsum")
                .build();
        var sessionId = "mySessionId";
        var application = new Application("1", "", "", sessionId, "", false, false, "", Collections.emptyList());
        var loadRequest = Load.builder()
                .sessionId(sessionId)
                .media(Media.builder()
                        .duration(originalVideoDuration)
                        .build())
                .build();
        var mediaStatus = TestMediaStatus.builder()
                .media(new su.litvak.chromecast.api.v2.Media(url, "", Double.MAX_VALUE, su.litvak.chromecast.api.v2.Media.StreamType.LIVE))
                .playerState(MediaStatus.PlayerState.PLAYING)
                .currentTime(1000.0)
                .build();
        var listener = mock(PlayerListener.class);
        var expectedResult = 35000000L;
        when(chromeCast.launchApp(ChromecastPlayer.MEDIA_RECEIVER_APP_ID)).thenReturn(application);
        when(chromeCast.getMediaStatus()).thenReturn(mediaStatus);
        when(service.toLoadRequest(sessionId, request)).thenReturn(loadRequest);
        when(service.toApplicationTime(originalVideoDuration)).thenReturn(expectedResult);
        player.addListener(listener);

        player.play(request);
        verify(chromeCast, timeout(1250)).getMediaStatus();

        verify(listener, timeout(250)).onDurationChanged(expectedResult);
    }

    @Test
    void testDispose_whenInvoked_shouldStopAppAndCloseConnection() throws IOException {
        var request = SimplePlayRequest.builder()
                .url("http://localhost/my-video.mp4")
                .title("lorem ipsum")
                .build();
        var application = mock(Application.class);
        var loadRequest = Load.builder()
                .sessionId("qwerty123")
                .media(Media.builder()
                        .duration(55000.00)
                        .build())
                .build();
        when(chromeCast.launchApp(ChromecastPlayer.MEDIA_RECEIVER_APP_ID)).thenReturn(application);
        when(service.toLoadRequest(any(), isA(PlayRequest.class))).thenReturn(loadRequest);
        player.play(request);

        verify(chromeCast, timeout(500)).send(isA(String.class), isA(Load.class));
        player.dispose();

        verify(chromeCast).stopApp();
        verify(chromeCast).disconnect();
    }
}
