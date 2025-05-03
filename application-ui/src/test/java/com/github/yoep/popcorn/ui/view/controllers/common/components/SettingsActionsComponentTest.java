package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.subtitles.ISubtitleService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.SuccessNotificationEvent;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class SettingsActionsComponentTest {
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
}