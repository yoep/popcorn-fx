package com.github.yoep.player.popcorn.controllers.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.player.popcorn.listeners.PopcornPlayerSectionListener;
import com.github.yoep.player.popcorn.messages.VideoMessage;
import com.github.yoep.player.popcorn.services.PopcornPlayerSectionService;
import com.github.yoep.player.popcorn.services.SubtitleManagerService;
import com.github.yoep.player.popcorn.subtitles.controls.SubtitleTrack;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import javafx.scene.control.Label;
import javafx.scene.input.ScrollEvent;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class PopcornPlayerSectionControllerTest {
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @Mock
    private PopcornPlayerSectionService sectionService;
    @Mock
    private SubtitleManagerService subtitleManagerService;
    @Mock
    private LocaleText localeText;
    @Mock
    private PlatformProvider platformProvider;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @InjectMocks
    private PopcornPlayerSectionController controller;

    private final AtomicReference<PopcornPlayerSectionListener> sectionListenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocation -> {
            sectionListenerHolder.set(invocation.getArgument(0, PopcornPlayerSectionListener.class));
            return null;
        }).when(sectionService).addListener(isA(PopcornPlayerSectionListener.class));
        lenient().doAnswer(invocation -> {
            invocation.getArgument(0, Runnable.class).run();
            return null;
        }).when(platformProvider).runOnRenderer(isA(Runnable.class));

        controller.playerPane = new Pane();
        controller.playerHeaderPane = new Pane();
        controller.playerControlsPane = new Pane();
        controller.infoLabel = new Label();
        controller.subtitleTrack = new SubtitleTrack();
    }

    @Test
    void testOnVolumeChanged_whenVolumeIsGiven_shouldShowVolumePercentage() {
        var volume = 80;
        var expectedResult = "Volume: 80%";
        when(localeText.get(VideoMessage.VIDEO_VOLUME, volume)).thenReturn(expectedResult);
        controller.initialize(url, resourceBundle);

        sectionListenerHolder.get().onVolumeChanged(volume);

        assertEquals(expectedResult, controller.infoLabel.getText());
    }

    @Test
    void testPlayerPane_whenPaneIsScrolledUp_shouldIncreaseTheVolume() {
        var deltaY = 15.0;
        var event = mock(ScrollEvent.class);
        when(event.getDeltaY()).thenReturn(deltaY);
        controller.initialize(url, resourceBundle);

        var eventHandler = controller.playerPane.getOnScroll();
        eventHandler.handle(event);

        verify(sectionService).onVolumeScroll(PopcornPlayerSectionController.VOLUME_INCREASE_AMOUNT);
    }

    @Test
    void testPlayerPane_whenPaneIsScrolledDown_shouldDecreaseTheVolume() {
        var deltaY = -40.0;
        var event = mock(ScrollEvent.class);
        when(event.getDeltaY()).thenReturn(deltaY);
        controller.initialize(url, resourceBundle);

        var eventHandler = controller.playerPane.getOnScroll();
        eventHandler.handle(event);

        verify(sectionService).onVolumeScroll(-PopcornPlayerSectionController.VOLUME_INCREASE_AMOUNT);
    }
}