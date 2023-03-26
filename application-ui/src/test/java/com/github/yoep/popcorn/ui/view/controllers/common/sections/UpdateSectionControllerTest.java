package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.ui.events.CloseUpdateEvent;
import com.github.yoep.popcorn.ui.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.popcorn.ui.view.services.UpdateSectionService;
import javafx.scene.control.Button;
import javafx.scene.image.ImageView;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.GridPane;
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
import java.util.concurrent.CompletableFuture;

import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class UpdateSectionControllerTest {
    @Mock
    private UpdateSectionService updateSectionService;
    @Mock
    private ImageService imageService;
    @Mock
    private LocaleText localeText;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private UpdateSectionController controller;

    @BeforeEach
    void setUp() {
        controller.updatePane = new GridPane();
        controller.updateNowButton = new Button();
        controller.logoImage = new ImageView();
        controller.backgroundCover = new BackgroundImageCover();
    }

    @Test
    void testOnUpdateNowClicked() {
        var event = mock(MouseEvent.class);
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        controller.initialize(url, resourceBundle);

        controller.onUpdateNowClicked(event);

        verify(event).consume();
        verify(updateSectionService).startUpdate();
    }

    @Test
    void testOnUpdateNowPressed() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        controller.initialize(url, resourceBundle);

        controller.onUpdateNowPressed(event);

        verify(event).consume();
        verify(updateSectionService).startUpdate();
    }

    @Test
    void testOnUpdatePressed() {
        var backSpaceEvent = mock(KeyEvent.class);
        var escapeEvent = mock(KeyEvent.class);
        when(backSpaceEvent.getCode()).thenReturn(KeyCode.BACK_SPACE);
        when(escapeEvent.getCode()).thenReturn(KeyCode.ESCAPE);
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        controller.initialize(url, resourceBundle);

        controller.onUpdatePressed(backSpaceEvent);
        verify(backSpaceEvent).consume();
        verify(eventPublisher).publish(new CloseUpdateEvent(controller));

        controller.onUpdatePressed(escapeEvent);
        verify(backSpaceEvent).consume();
        verify(eventPublisher, times(2)).publish(new CloseUpdateEvent(controller));
    }
}