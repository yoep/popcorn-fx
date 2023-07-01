package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
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
    private SubtitleService subtitleService;
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
}