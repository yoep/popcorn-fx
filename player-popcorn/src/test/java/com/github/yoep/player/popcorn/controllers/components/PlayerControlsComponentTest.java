package com.github.yoep.player.popcorn.controllers.components;

import com.github.yoep.player.popcorn.controls.ProgressSliderControl;
import com.github.yoep.player.popcorn.listeners.PlayerControlsListener;
import com.github.yoep.player.popcorn.services.PlayerControlsService;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.adapters.torrent.model.DownloadStatus;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.LongProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.beans.property.SimpleLongProperty;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;

import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PlayerControlsComponentTest {
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
    private final BooleanProperty valueChangingProperty= new SimpleBooleanProperty();
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
}