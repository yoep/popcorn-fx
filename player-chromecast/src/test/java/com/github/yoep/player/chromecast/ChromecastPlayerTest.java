package com.github.yoep.player.chromecast;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import su.litvak.chromecast.api.v2.ChromeCast;

import static org.junit.jupiter.api.Assertions.assertEquals;
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
}
