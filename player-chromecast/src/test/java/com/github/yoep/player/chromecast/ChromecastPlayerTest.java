package com.github.yoep.player.chromecast;

import com.github.yoep.player.chromecast.api.v2.Load;
import com.github.yoep.player.chromecast.services.ChromecastService;
import com.github.yoep.popcorn.backend.adapters.player.PlayRequest;
import com.github.yoep.popcorn.backend.player.model.SimplePlayRequest;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import su.litvak.chromecast.api.v2.Application;
import su.litvak.chromecast.api.v2.ChromeCast;

import java.io.IOException;
import java.util.Collections;

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
    void testStop_whenInvoked_shouldStopTheChromecastPlayer() throws IOException, InterruptedException {
        player.stop();
        Thread.sleep(50);

        verify(chromeCast).stopApp();
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
                .build();
        when(chromeCast.launchApp(ChromecastPlayer.MEDIA_RECEIVER_APP_ID)).thenReturn(application);
        when(service.toLoadRequest(sessionId, request)).thenReturn(loadRequest);

        player.play(request);

        verify(chromeCast, timeout(500)).send(ChromecastPlayer.MEDIA_NAMESPACE, loadRequest);
    }

    @Test
    void testDispose_whenInvoked_shouldStopAppAndCloseConnection() throws IOException {
        var request = SimplePlayRequest.builder()
                .url("http://localhost/my-video.mp4")
                .title("lorem ipsum")
                .build();
        var application = mock(Application.class);
        var loadRequest = mock(Load.class);
        when(chromeCast.launchApp(ChromecastPlayer.MEDIA_RECEIVER_APP_ID)).thenReturn(application);
        when(service.toLoadRequest(any(), isA(PlayRequest.class))).thenReturn(loadRequest);
        player.play(request);

        verify(chromeCast, timeout(500)).send(isA(String.class), isA(Load.class));
        player.dispose();

       verify(chromeCast).stopApp();
       verify(chromeCast).disconnect();
    }
}
