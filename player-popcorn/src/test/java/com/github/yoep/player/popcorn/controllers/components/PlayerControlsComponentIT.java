package com.github.yoep.player.popcorn.controllers.components;

import com.github.yoep.player.popcorn.controls.ProgressSliderControl;
import com.github.yoep.player.popcorn.controls.Volume;
import com.github.yoep.player.popcorn.listeners.PlayerControlsListener;
import com.github.yoep.player.popcorn.services.PlayerControlsService;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.LongProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.beans.property.SimpleLongProperty;
import javafx.scene.control.Label;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class PlayerControlsComponentIT {
    @Mock
    private PlayerControlsService playerControlsService;
    @Mock
    private PlatformProvider platformProvider;
    @Mock
    private URL location;
    @Mock
    private ResourceBundle resources;
    @Mock
    private ProgressSliderControl playProgress;
    @InjectMocks
    private PlayerControlsComponent component;

    private final AtomicReference<PlayerControlsListener> listenerHolder = new AtomicReference<>();
    private final BooleanProperty valueChangingProperty = new SimpleBooleanProperty();
    private final LongProperty timeProperty = new SimpleLongProperty();

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocationOnMock -> {
            var runnable = invocationOnMock.getArgument(0, Runnable.class);
            runnable.run();
            return null;
        }).when(platformProvider).runOnRenderer(isA(Runnable.class));
        lenient().doAnswer(invocationOnMock -> {
            listenerHolder.set(invocationOnMock.getArgument(0, PlayerControlsListener.class));
            return null;
        }).when(playerControlsService).addListener(isA(PlayerControlsListener.class));
        when(playProgress.valueChangingProperty()).thenReturn(valueChangingProperty);
        when(playProgress.timeProperty()).thenReturn(timeProperty);

        component.playProgress = playProgress;
        component.volumeIcon = new Volume();
        component.durationLabel = new Label();
        component.timeLabel = new Label();
    }

    @Test
    void testPlayerControlsListener_whenDownloadStatusChanged_shouldUpdateThePlayProgressLoadStatus() {
        var progress = 0.6f;
        var downloadStatus = mock(DownloadStatus.class);
        when(downloadStatus.getProgress()).thenReturn(progress);
        component.initialize(location, resources);

        listenerHolder.get().onDownloadStatusChanged(downloadStatus);

        verify(playProgress).setLoadProgress(progress);
    }

    @Test
    void testVolumeChanged_whenVolumeIsChanged_shouldUpdatePlayerVolume() {
        var expectedResult = 20.0;
        component.initialize(location, resources);

        component.volumeIcon.setVolume(expectedResult);

        verify(playerControlsService).onVolumeChanged(expectedResult);
    }

    @Test
    void testPlayerListener_whenVolumeIsChanged_shouldUpdateVolumeIcon() {
        var volume = 30;
        var expectedResult = 0.3;
        component.initialize(location, resources);

        listenerHolder.get().onVolumeChanged(volume);

        assertEquals(expectedResult, component.volumeIcon.getVolume());
    }

    @Test
    void testPlayerListener_whenDurationIsChanged_shouldUpdateDurationLabel() {
        component.initialize(location, resources);

        listenerHolder.get().onPlayerDurationChanged(1200000);

        assertEquals("20:00", component.durationLabel.getText());
    }

    @Test
    void testPlayerListener_whenTimeIsChanged_shouldUpdateTimeLabel() {
        component.initialize(location, resources);

        listenerHolder.get().onPlayerTimeChanged(100000);

        assertEquals("01:40", component.timeLabel.getText());
    }
}