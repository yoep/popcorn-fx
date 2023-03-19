package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.OverlayItemListener;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.OverlayItemMetadataProvider;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class OverlayMediaCardComponentTest {
    @Mock
    private LocaleText localeText;
    @Mock
    private ImageService imageService;
    @Mock
    private OverlayItemMetadataProvider metadataProvider;
    @Mock
    private Media media;

    @Test
    void testOnEnterPressed() {
        var event = mock(KeyEvent.class);
        var listener = mock(OverlayItemListener.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        var component = new DesktopMediaCardComponent(media, localeText, imageService, metadataProvider, listener);

        component.onKeyPressed(event);

        verify(listener).onClicked(media);
    }
}