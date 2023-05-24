package com.github.yoep.player.popcorn.controllers.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.player.popcorn.services.PlayerControlsService;
import com.github.yoep.player.popcorn.services.PlayerSubtitleService;
import com.github.yoep.popcorn.backend.events.ClosePlayerEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.messages.SubtitleMessage;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import javafx.event.Event;
import javafx.scene.Scene;
import javafx.scene.control.Button;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.AnchorPane;
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
import java.util.Objects;
import java.util.ResourceBundle;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class TvPlayerControlsComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private PlayerControlsService playerControlsService;
    @Mock
    private PlayerSubtitleService subtitleService;
    @Mock
    private LocaleText localeText;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private TvPlayerControlsComponent component;

    @BeforeEach
    void setUp() {
        component.playButton = new Icon();
        component.subtitleOverlay = new Overlay();
        component.subtitleSelection = new AxisItemSelection<>();
        component.subtitleIncreaseOffset = new Button();
        component.subtitleDecreaseOffset = new Button();

        component.subtitleOverlay.getChildren().add(component.subtitleSelection);
    }

    @Test
    void testInitializeText() throws TimeoutException {
        var increaseText = "lorem";
        var decreaseText = "ipsum";
        when(localeText.get(SubtitleMessage.INCREASE_SUBTITLE_OFFSET, TvPlayerControlsComponent.OFFSET_IN_SECONDS)).thenReturn(increaseText);
        when(localeText.get(SubtitleMessage.DECREASE_SUBTITLE_OFFSET, TvPlayerControlsComponent.OFFSET_IN_SECONDS)).thenReturn(decreaseText);

        component.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitForFxEvents();

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> Objects.equals(component.subtitleIncreaseOffset.getText(), increaseText));
        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> Objects.equals(component.subtitleDecreaseOffset.getText(), decreaseText));
    }

    @Test
    void testOnPlayClicked() {
        var event = mock(MouseEvent.class);

        component.onPlayClicked(event);

        verify(event).consume();
        verify(playerControlsService).togglePlayerPlaybackState();
    }

    @Test
    void testOnPlayPressed() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);

        component.onPlayPressed(event);

        verify(event).consume();
        verify(playerControlsService).togglePlayerPlaybackState();
    }

    @Test
    void testOnStopClicked() {
        var event = mock(MouseEvent.class);

        component.onStopClicked(event);

        verify(event).consume();
        verify(eventPublisher).publish(new ClosePlayerEvent(component, ClosePlayerEvent.Reason.USER));
    }

    @Test
    void testOnStopPressed() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);

        component.onStopPressed(event);

        verify(event).consume();
        verify(eventPublisher).publish(new ClosePlayerEvent(component, ClosePlayerEvent.Reason.USER));
    }

    @Test
    void testOnIncreaseFontSize() {
        var event = mock(Event.class);

        component.onIncreaseFontSize(event);

        verify(event).consume();
        verify(subtitleService).updateSubtitleSizeWithSizeOffset(4);
    }

    @Test
    void testOnDecreaseFontSize() {
        var event = mock(Event.class);

        component.onDecreaseFontSize(event);

        verify(event).consume();
        verify(subtitleService).updateSubtitleSizeWithSizeOffset(-4);
    }

    @Test
    void testOnSubtitle() {
        var event = mock(Event.class);
        var parent = new AnchorPane(component.subtitleOverlay);
        var scene = new Scene(parent);
        component.initialize(url, resourceBundle);

        component.onChangeSubtitle(event);

        verify(playerControlsService).pause();
    }
}