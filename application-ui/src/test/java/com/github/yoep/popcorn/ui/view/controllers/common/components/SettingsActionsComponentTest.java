package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ResetProviderApiRequest;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.SuccessNotificationEvent;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseButton;
import javafx.scene.input.MouseEvent;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReferenceArray;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class SettingsActionsComponentTest {
    @Mock
    private FxChannel fxChannel;
    @Mock
    private ISubtitleService subtitleService;
    @Mock
    private TorrentService torrentService;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private LocaleText localeText;
    @InjectMocks
    private SettingsActionsComponent component;

    @Test
    void testOnCleanSubtitleClicked() {
        var text = "lorem";
        var event = mock(MouseEvent.class);
        when(event.getButton()).thenReturn(MouseButton.PRIMARY);
        when(localeText.get(SettingsActionsComponent.SUBTITLES_CLEANED_MESSAGE)).thenReturn(text);

        component.onCleanSubtitlesClicked(event);

        verify(event).consume();
        verify(subtitleService).cleanup();
        verify(eventPublisher).publish(new SuccessNotificationEvent(component, text));
    }

    @Test
    void testOnCleanSubtitlePressed() {
        var text = "lorem";
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        when(localeText.get(SettingsActionsComponent.SUBTITLES_CLEANED_MESSAGE)).thenReturn(text);

        component.onCleanSubtitlesPressed(event);

        verify(event).consume();
        verify(subtitleService).cleanup();
        verify(eventPublisher).publish(new SuccessNotificationEvent(component, text));
    }

    @Test
    void testOnCleanTorrentsClicked() {
        var text = "ipsum";
        var event = mock(MouseEvent.class);
        when(event.getButton()).thenReturn(MouseButton.PRIMARY);
        when(localeText.get(SettingsActionsComponent.TORRENTS_CLEANED_MESSAGE)).thenReturn(text);

        component.onCleanTorrentsClicked(event);

        verify(event).consume();
        verify(torrentService).cleanup();
        verify(eventPublisher).publish(new SuccessNotificationEvent(component, text));
    }

    @Test
    void testOnCleanTorrentsPressed() {
        var text = "ipsum";
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        when(localeText.get(SettingsActionsComponent.TORRENTS_CLEANED_MESSAGE)).thenReturn(text);

        component.onCleanTorrentsPressed(event);

        verify(event).consume();
        verify(torrentService).cleanup();
        verify(eventPublisher).publish(new SuccessNotificationEvent(component, text));
    }

    @Test
    void testOnResetApiClicked() {
        var text = "reset complete";
        var event = mock(MouseEvent.class);
        var holders = new AtomicReferenceArray<ResetProviderApiRequest>(2);
        var holderIndex = new AtomicInteger(0);
        when(event.getButton()).thenReturn(MouseButton.PRIMARY);
        when(localeText.get(SettingsActionsComponent.RESET_API_MESSAGE)).thenReturn(text);
        doAnswer(invocation -> {
            holders.set(holderIndex.getAndIncrement(), invocation.getArgument(0, ResetProviderApiRequest.class));
            return null;
        }).when(fxChannel).send(isA(ResetProviderApiRequest.class));

        component.onResetApiClicked(event);

        verify(event).consume();
        verify(fxChannel, times(2)).send(isA(ResetProviderApiRequest.class));
        verify(eventPublisher).publish(new SuccessNotificationEvent(component, text));
        // verify that both movies and series have been reset
        var request1 = holders.get(0);
        assertEquals(Media.Category.MOVIES, request1.getCategory());
        var request2 = holders.get(1);
        assertEquals(Media.Category.SERIES, request2.getCategory());
    }

    @Test
    void testOnResetApiPressed() {
        var text = "reset complete";
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        when(localeText.get(SettingsActionsComponent.RESET_API_MESSAGE)).thenReturn(text);

        component.onResetApiPressed(event);

        verify(event).consume();
        verify(fxChannel, times(2)).send(isA(ResetProviderApiRequest.class));
        verify(eventPublisher).publish(new SuccessNotificationEvent(component, text));
    }
}