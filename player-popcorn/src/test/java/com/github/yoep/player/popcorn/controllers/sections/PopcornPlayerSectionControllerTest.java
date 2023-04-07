package com.github.yoep.player.popcorn.controllers.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.player.popcorn.listeners.PopcornPlayerSectionListener;
import com.github.yoep.player.popcorn.messages.VideoMessage;
import com.github.yoep.player.popcorn.services.PopcornPlayerSectionService;
import com.github.yoep.player.popcorn.services.SubtitleManagerService;
import com.github.yoep.player.popcorn.subtitles.controls.SubtitleTrack;
import com.github.yoep.popcorn.backend.adapters.player.state.PlayerState;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayVideoEvent;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import javafx.animation.Animation;
import javafx.scene.control.Label;
import javafx.scene.input.ScrollEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import org.apache.commons.lang3.StringUtils;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicReference;

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
    private ViewLoader viewLoader;
    @Mock
    private ApplicationConfig applicationConfig;
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

        controller.playerPane = new AnchorPane();
        controller.playerHeaderPane = new Pane();
        controller.playerControlsPane = new Pane();
        controller.playerVideoOverlay = new Pane();
        controller.bufferPane = new Pane();
        controller.infoLabel = new Label();
        controller.errorText = new Label();
        controller.subtitleTrack = new SubtitleTrack();
    }

    @Test
    void testOnVolumeChanged_whenVolumeIsGiven_shouldShowVolumePercentage() throws TimeoutException {
        var volume = 80;
        var expectedResult = "Volume: 80%";
        when(localeText.get(VideoMessage.VIDEO_VOLUME, volume)).thenReturn(expectedResult);
        when(viewLoader.load(PopcornPlayerSectionController.VIEW_CONTROLS)).thenReturn(new Pane());
        controller.initialize(url, resourceBundle);

        sectionListenerHolder.get().onVolumeChanged(volume);

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> controller.infoLabel.getText().equals(expectedResult));
    }

    @Test
    void testPlayerPane_whenPaneIsScrolledUp_shouldIncreaseTheVolume() {
        var deltaY = 15.0;
        var event = mock(ScrollEvent.class);
        when(event.getDeltaY()).thenReturn(deltaY);
        when(viewLoader.load(PopcornPlayerSectionController.VIEW_CONTROLS)).thenReturn(new Pane());
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
        when(viewLoader.load(PopcornPlayerSectionController.VIEW_CONTROLS)).thenReturn(new Pane());
        controller.initialize(url, resourceBundle);

        var eventHandler = controller.playerPane.getOnScroll();
        eventHandler.handle(event);

        verify(sectionService).onVolumeScroll(-PopcornPlayerSectionController.VOLUME_INCREASE_AMOUNT);
    }

    @Test
    void testOnPlayVideoEvent() throws TimeoutException {
        when(viewLoader.load(PopcornPlayerSectionController.VIEW_CONTROLS)).thenReturn(new Pane());
        controller.initialize(url, resourceBundle);

        controller.errorText.setText("Lorem");
        eventPublisher.publish(new PlayVideoEvent(this, "http://localhost/video.mp4", "Lorem ipsum", false));
        WaitForAsyncUtils.waitForFxEvents();

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> StringUtils.isEmpty(controller.errorText.getText()));
    }

    @Test
    void testTvMode() throws TimeoutException {
        when(applicationConfig.isTvMode()).thenReturn(true);
        when(viewLoader.load(PopcornPlayerSectionController.VIEW_CONTROLS)).thenReturn(new Pane());

        controller.initialize(url, resourceBundle);

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS,
                () -> AnchorPane.getTopAnchor(controller.infoLabel) == PopcornPlayerSectionController.INFO_TOP_TV_MODE);
    }

    @Test
    void testOnPlaying() throws TimeoutException {
        when(viewLoader.load(PopcornPlayerSectionController.VIEW_CONTROLS)).thenReturn(new Pane());
        controller.initialize(url, resourceBundle);

        var listener = sectionListenerHolder.get();
        listener.onPlayerStateChanged(PlayerState.PLAYING);

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> controller.bufferPane.getChildren().size() == 0);
        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> controller.idleTimer.getStatus() == Animation.Status.RUNNING);
    }
}