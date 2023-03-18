package com.github.yoep.popcorn.ui.view.controllers.desktop.sections;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.ui.events.CloseAboutEvent;
import com.github.yoep.popcorn.ui.view.controls.ImageCover;
import com.github.yoep.popcorn.ui.view.services.AboutSectionService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.scene.control.Label;
import javafx.scene.image.ImageView;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.ApplicationContext;
import org.testfx.framework.junit5.ApplicationExtension;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class AboutSectionControllerTest {
    @Mock
    private ApplicationContext applicationContext;
    @Mock
    private AboutSectionService aboutService;
    @Mock
    private ImageService imageService;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private FxLib fxLib;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private AboutSectionController controller;

    @BeforeEach
    void setUp() {
        controller.titleLabel = new Label("titleLabel");
        controller.versionLabel = new Label("versionLabel");
        controller.backgroundCover = new ImageCover();
        controller.logoImage = new ImageView();
    }

    @Test
    void testInitialize() {
        var version = "10.0.0";
        when(fxLib.version()).thenReturn(version);
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());

        controller.initialize(url, resourceBundle);

        assertEquals(version, controller.versionLabel.getText());
    }

    @Test
    void testOnAboutPressed() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.BACK_SPACE);
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        controller.initialize(url, resourceBundle);

        controller.onAboutPressed(event);

        verify(event).consume();
        verify(eventPublisher).publish(new CloseAboutEvent(controller));
    }
}