package com.github.yoep.player.chromecast;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import su.litvak.chromecast.api.v2.ChromeCast;

import java.io.IOException;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class ChromecastPlayerTest {
    @Mock
    private ChromeCast chromeCast;
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

        verify(chromeCast).stopApp();
    }

    @Test
    void testSeek_whenTimeIsGiven_shouldSeekChromecastPlayer() throws IOException {
        var time = 845500L;

        player.seek(time);

        verify(chromeCast).seek(time);
    }

    @Test
    void testVolume_whenVolumeIsGiven_shouldChangeChromecastVolume() throws IOException {
        var volume = 50;

        player.volume(volume);

        verify(chromeCast).setVolume(0.5f);
    }
}
