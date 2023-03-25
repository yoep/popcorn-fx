package com.github.yoep.popcorn.ui.view.controllers.desktop.sections;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.updater.UpdateCallback;
import com.github.yoep.popcorn.backend.updater.UpdateCallbackEvent;
import com.github.yoep.popcorn.backend.updater.UpdateService;
import com.github.yoep.popcorn.backend.updater.UpdateState;
import com.github.yoep.popcorn.ui.events.CloseAboutEvent;
import com.github.yoep.popcorn.ui.messages.UpdateMessage;
import com.github.yoep.popcorn.ui.view.controls.ImageCover;
import com.github.yoep.popcorn.ui.view.services.AboutSectionService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.scene.control.Button;
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
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class AboutSectionControllerTest {
    @Mock
    private AboutSectionService aboutService;
    @Mock
    private ImageService imageService;
    @Mock
    private UpdateService updateService;
    @Mock
    private LocaleText localeText;
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

    private final AtomicReference<UpdateCallback> callbackHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocation -> {
            callbackHolder.set(invocation.getArgument(0));
            return null;
        }).when(updateService).register(isA(UpdateCallback.class));

        controller.versionLabel = new Label("versionLabel");
        controller.backgroundCover = new ImageCover();
        controller.logoImage = new ImageView();
        controller.updateButton = new Button();
        controller.updateIcon = new Icon();
    }

    @Test
    void testInitialize() {
        var version = "10.0.0";
        when(fxLib.version()).thenReturn(version);
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        when(updateService.getState()).thenReturn(UpdateState.NO_UPDATE_AVAILABLE);

        controller.initialize(url, resourceBundle);

        assertEquals(version, controller.versionLabel.getText());
    }

    @Test
    void testOnAboutPressed() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.BACK_SPACE);
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        when(updateService.getState()).thenReturn(UpdateState.NO_UPDATE_AVAILABLE);
        controller.initialize(url, resourceBundle);

        controller.onAboutPressed(event);

        verify(event).consume();
        verify(eventPublisher).publish(new CloseAboutEvent(controller));
    }

    @Test
    void testUpdateButtonNoUpdateAvailable() throws TimeoutException {
        var buttonText = "Check for updates";
        when(updateService.getState()).thenReturn(UpdateState.NO_UPDATE_AVAILABLE);
        when(localeText.get(UpdateMessage.CHECK_FOR_NEW_UPDATES)).thenReturn(buttonText);
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        controller.initialize(url, resourceBundle);

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> controller.updateButton.getText().equals(buttonText));
        assertEquals(Icon.REFRESH_UNICODE, controller.updateIcon.getText());
    }

    @Test
    void testUpdateButtonUpdateAvailable() throws TimeoutException {
        var buttonText = "update available";
        when(updateService.getState()).thenReturn(UpdateState.UPDATE_AVAILABLE);
        when(localeText.get(UpdateMessage.DOWNLOAD_UPDATE)).thenReturn(buttonText);
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        controller.initialize(url, resourceBundle);

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> controller.updateButton.getText().equals(buttonText));
        assertEquals(Icon.DOWNLOAD_UNICODE, controller.updateIcon.getText());
    }

    @Test
    void testUpdateButtonUpdateError() throws TimeoutException {
        var buttonText = "error";
        when(updateService.getState()).thenReturn(UpdateState.ERROR);
        when(localeText.get(UpdateMessage.NO_UPDATE_AVAILABLE)).thenReturn(buttonText);
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        controller.initialize(url, resourceBundle);

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> controller.updateButton.getText().equals(buttonText));
        assertEquals(Icon.TIMES_UNICODE, controller.updateIcon.getText());
    }

    @Test
    void testOnUpdateStateChanged() throws TimeoutException {
        var event = new UpdateCallbackEvent.ByValue();
        event.tag = UpdateCallbackEvent.Tag.StateChanged;
        event.union = new UpdateCallbackEvent.UpdateEventCUnion.ByValue();
        event.union.state_changed = new UpdateCallbackEvent.StateChangedBody();
        event.union.state_changed.newState = UpdateState.UPDATE_AVAILABLE;
        when(updateService.getState()).thenReturn(UpdateState.ERROR);
        when(localeText.get(UpdateMessage.NO_UPDATE_AVAILABLE)).thenReturn("error");
        when(localeText.get(UpdateMessage.DOWNLOAD_UPDATE)).thenReturn("lorem");
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        controller.initialize(url, resourceBundle);

        var callback = callbackHolder.get();
        callback.callback(event);

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> controller.updateButton.getText().equals("lorem"));
        assertEquals(Icon.DOWNLOAD_UNICODE, controller.updateIcon.getText());
    }
}