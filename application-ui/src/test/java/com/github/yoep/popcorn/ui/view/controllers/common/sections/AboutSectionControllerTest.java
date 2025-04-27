package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.messages.UpdateMessage;
import com.github.yoep.popcorn.backend.updater.UpdateEventListener;
import com.github.yoep.popcorn.backend.updater.UpdateService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.CloseAboutEvent;
import com.github.yoep.popcorn.ui.events.ShowUpdateEvent;
import com.github.yoep.popcorn.ui.font.controls.Icon;
import com.github.yoep.popcorn.ui.view.controls.ImageCover;
import com.github.yoep.popcorn.ui.view.services.AboutSectionService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.google.protobuf.Parser;
import javafx.scene.control.Button;
import javafx.scene.control.Label;
import javafx.scene.image.ImageView;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
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
import java.util.Optional;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class AboutSectionControllerTest {
    static final String VERSION_VALUE = "10.0.0";

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
    private FxChannel fxChannel;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    private AboutSectionController controller;

    private final AtomicReference<UpdateEventListener> updateEventListener = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocation -> {
            updateEventListener.set(invocation.getArgument(0, UpdateEventListener.class));
            return null;
        }).when(updateService).addListener(isA(UpdateEventListener.class));
        when(fxChannel.send(isA(GetApplicationVersionRequest.class), isA(Parser.class))).thenReturn(CompletableFuture.completedFuture(GetApplicationVersionResponse.newBuilder()
                .setVersion(VERSION_VALUE)
                .build()));

        controller = new AboutSectionController(aboutService, imageService, eventPublisher, updateService, localeText, fxChannel);

        controller.versionLabel = new Label("versionLabel");
        controller.newVersionLabel = new Label("newVersionLabel");
        controller.backgroundCover = new ImageCover();
        controller.logoImage = new ImageView();
        controller.updateButton = new Button();
        controller.updateIcon = new Icon();
    }

    @Test
    void testInitialize() {
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        when(updateService.getState()).thenReturn(CompletableFuture.completedFuture(Update.State.NO_UPDATE_AVAILABLE));

        controller.initialize(url, resourceBundle);

        assertEquals(VERSION_VALUE, controller.versionLabel.getText());
    }

    @Test
    void testOnAboutPressed() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.BACK_SPACE);
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        when(updateService.getState()).thenReturn(CompletableFuture.completedFuture(Update.State.NO_UPDATE_AVAILABLE));
        controller.initialize(url, resourceBundle);

        controller.onAboutPressed(event);

        verify(event).consume();
        verify(eventPublisher).publish(new CloseAboutEvent(controller));
    }

    @Test
    void testUpdateButtonNoUpdateAvailable() throws TimeoutException {
        var buttonText = "Check for updates";
        when(updateService.getState()).thenReturn(CompletableFuture.completedFuture(Update.State.NO_UPDATE_AVAILABLE));
        when(localeText.get(UpdateMessage.CHECK_FOR_NEW_UPDATES)).thenReturn(buttonText);
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        controller.initialize(url, resourceBundle);

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> controller.updateButton.getText().equals(buttonText));
        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> controller.updateIcon.getText().equals(Icon.REFRESH_UNICODE));
    }

    @Test
    void testUpdateButtonUpdateAvailable() throws TimeoutException {
        var buttonText = "update available";
        when(updateService.getState()).thenReturn(CompletableFuture.completedFuture(Update.State.UPDATE_AVAILABLE));
        when(updateService.getUpdateInfo()).thenReturn(CompletableFuture.completedFuture(Optional.of(Update.VersionInfo.newBuilder()
                .setApplication(Update.PatchInfo.newBuilder()
                        .setVersion("1.0.0")
                        .build())
                .setRuntime(Update.PatchInfo.newBuilder()
                        .setVersion("2.0.0")
                        .build())
                .build())));
        when(localeText.get(UpdateMessage.DOWNLOAD_UPDATE)).thenReturn(buttonText);
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        controller.initialize(url, resourceBundle);

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> controller.updateButton.getText().equals(buttonText));
        assertEquals(Icon.DOWNLOAD_UNICODE, controller.updateIcon.getText());
    }

    @Test
    void testUpdateButtonUpdateError() throws TimeoutException {
        var buttonText = "error";
        when(updateService.getState()).thenReturn(CompletableFuture.completedFuture(Update.State.ERROR));
        when(localeText.get(UpdateMessage.NO_UPDATE_AVAILABLE)).thenReturn(buttonText);
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        controller.initialize(url, resourceBundle);

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> controller.updateButton.getText().equals(buttonText));
        assertEquals(Icon.TIMES_UNICODE, controller.updateIcon.getText());
    }

    @Test
    void testOnUpdateClickedAndStateIsNoUpdateAvailable() {
        var event = mock(MouseEvent.class);
        when(updateService.getState()).thenReturn(CompletableFuture.completedFuture(Update.State.NO_UPDATE_AVAILABLE));
        when(localeText.get(UpdateMessage.CHECK_FOR_NEW_UPDATES)).thenReturn("lorem");
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        controller.initialize(url, resourceBundle);

        controller.onUpdateClicked(event);

        verify(event).consume();
        verify(updateService).checkForUpdates();
    }

    @Test
    void testOnUpdatePressedAndStateIsUpdateAvailable() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        when(updateService.getState()).thenReturn(CompletableFuture.completedFuture(Update.State.UPDATE_AVAILABLE));
        when(updateService.getUpdateInfo()).thenReturn(CompletableFuture.completedFuture(Optional.of(Update.VersionInfo.newBuilder()
                .setApplication(Update.PatchInfo.newBuilder()
                        .setVersion("1.0.0")
                        .build())
                .setRuntime(Update.PatchInfo.newBuilder()
                        .setVersion("2.0.0")
                        .build())
                .build())));
        when(localeText.get(UpdateMessage.DOWNLOAD_UPDATE)).thenReturn("ipsum");
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        controller.initialize(url, resourceBundle);

        controller.onUpdatePressed(event);

        verify(event).consume();
        verify(eventPublisher).publish(new ShowUpdateEvent(controller));
    }

    @Test
    void testOnUpdateStateChanged() {
        when(localeText.get(UpdateMessage.CHECKING_FOR_UPDATES)).thenReturn("Checking");
        when(localeText.get(UpdateMessage.DOWNLOAD_UPDATE)).thenReturn("Download");
        when(localeText.get(UpdateMessage.CHECK_FOR_NEW_UPDATES)).thenReturn("Checking");
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        when(updateService.getState()).thenReturn(CompletableFuture.completedFuture(Update.State.NO_UPDATE_AVAILABLE));
        when(updateService.getUpdateInfo()).thenReturn(new CompletableFuture<>());
        controller.initialize(url, resourceBundle);

        updateEventListener.get().onStateChanged(UpdateEvent.StateChanged.newBuilder()
                .setNewState(Update.State.CHECKING_FOR_NEW_VERSION)
                .build());
        WaitForAsyncUtils.waitForFxEvents();
        assertEquals("Checking", controller.updateButton.getText());

        updateEventListener.get().onStateChanged(UpdateEvent.StateChanged.newBuilder()
                .setNewState(Update.State.UPDATE_AVAILABLE)
                .build());
        WaitForAsyncUtils.waitForFxEvents();
        assertEquals("Download", controller.updateButton.getText());
    }
}