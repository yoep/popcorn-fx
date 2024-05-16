package com.github.yoep.player.popcorn.controls;

import com.github.yoep.popcorn.ui.font.controls.IconSolid;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import static org.junit.jupiter.api.Assertions.assertEquals;

@ExtendWith({ApplicationExtension.class})
class VolumeTest {
    private Volume volume;

    @BeforeEach
    void setUp() {
        volume = new Volume();
    }

    @Test
    void testVolumeChanged_whenVolumeIsZero_shouldUpdateIconToMuted() {
        volume.setVolume(10.0);

        volume.setVolume(0.0);
        WaitForAsyncUtils.waitForFxEvents(10);

        assertEquals(IconSolid.VOLUME_MUTE_UNICODE, volume.getText());
    }

    @Test
    void testVolumeChanged_whenVolumeIs100_shouldUpdateIconToMax() {
        volume.setVolume(100.0);
        WaitForAsyncUtils.waitForFxEvents(10);

        assertEquals(IconSolid.VOLUME_UP_UNICODE, volume.getText());
    }
}