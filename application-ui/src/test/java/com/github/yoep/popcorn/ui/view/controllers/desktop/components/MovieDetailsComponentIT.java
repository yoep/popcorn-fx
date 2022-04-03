package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.view.listeners.DetailsComponentListener;
import com.github.yoep.popcorn.ui.view.services.DetailsComponentService;
import javafx.scene.control.Tooltip;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.doAnswer;
import static org.mockito.Mockito.when;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class MovieDetailsComponentIT {
    @Mock
    private DetailsComponentService service;
    @Mock
    private LocaleText localeText;
    @InjectMocks
    private MovieDetailsComponent component;

    private final AtomicReference<DetailsComponentListener> listenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, DetailsComponentListener.class));
            return null;
        }).when(service).addListener(isA(DetailsComponentListener.class));

        component.watchedIcon = new Icon();
        component.watchedTooltip = new Tooltip();
        component.favoriteIcon = new Icon();
        component.favoriteTooltip = new Tooltip();
    }

    @Test
    void testListeners_whenWatchedStateIsChangedToTrue_shouldUpdateWatchedIcon() {
        var message = "watched";
        when(localeText.get(DetailsMessage.MARK_AS_NOT_SEEN)).thenReturn(message);
        component.init();

        var listener = listenerHolder.get();
        listener.onWatchChanged(true);
        WaitForAsyncUtils.waitForFxEvents(10);

        assertEquals(Icon.CHECK_UNICODE, component.watchedIcon.getText());
        assertEquals(message, component.watchedTooltip.getText());
    }

    @Test
    void testListeners_whenWatchedStateIsChangedToFalse_shouldUpdateWatchedIcon() {
        var message = "not seen";
        when(localeText.get(DetailsMessage.MARK_AS_SEEN)).thenReturn(message);
        component.init();

        var listener = listenerHolder.get();
        listener.onWatchChanged(false);
        WaitForAsyncUtils.waitForFxEvents(10);

        assertEquals(Icon.EYE_SLASH_UNICODE, component.watchedIcon.getText());
        assertEquals(message, component.watchedTooltip.getText());
    }

    @Test
    void testListeners_whenLikedStateIsChangedToTrue_shouldUpdateLikedIcon() {
        var message = "liked";
        when(localeText.get(DetailsMessage.REMOVE_FROM_BOOKMARKS)).thenReturn(message);
        component.init();

        var listener = listenerHolder.get();
        listener.onLikedChanged(true);
        WaitForAsyncUtils.waitForFxEvents(10);

        assertEquals(Icon.HEART_UNICODE, component.favoriteIcon.getText());
        assertEquals(message, component.favoriteTooltip.getText());
    }
}