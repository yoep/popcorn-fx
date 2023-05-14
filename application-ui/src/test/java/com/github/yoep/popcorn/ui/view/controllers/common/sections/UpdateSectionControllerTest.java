package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.updater.*;
import com.github.yoep.popcorn.ui.events.CloseUpdateEvent;
import com.github.yoep.popcorn.ui.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.GridPane;
import javafx.scene.layout.Pane;
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

import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class UpdateSectionControllerTest {
    @Mock
    private UpdateService updateService;
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
        controller.backgroundCover = new BackgroundImageCover();
        controller.progressPane = new Pane();
        controller.progressLabel = new Label();
        controller.progressBar = new ProgressBar();
    }

    @Test
    void testOnUpdatePressed() {
        var backSpaceEvent = mock(KeyEvent.class);
        var escapeEvent = mock(KeyEvent.class);
        when(backSpaceEvent.getCode()).thenReturn(KeyCode.BACK_SPACE);
        when(escapeEvent.getCode()).thenReturn(KeyCode.ESCAPE);
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        when(updateService.getState()).thenReturn(UpdateState.UPDATE_AVAILABLE);
        controller.initialize(url, resourceBundle);

        controller.onUpdatePressed(backSpaceEvent);
        verify(backSpaceEvent).consume();
        verify(eventPublisher).publish(new CloseUpdateEvent(controller));

        controller.onUpdatePressed(escapeEvent);
        verify(backSpaceEvent).consume();
        verify(eventPublisher, times(2)).publish(new CloseUpdateEvent(controller));
    }

    @Test
    void testOnDownloadProgress() throws TimeoutException {
        var listenerHolder = new AtomicReference<UpdateCallback>();
        var event = new UpdateCallbackEvent.ByValue();
        event.tag = UpdateCallbackEvent.Tag.DownloadProgress;
        event.union = new UpdateCallbackEvent.UpdateEventCUnion.ByValue();
        event.union.download_progress = new UpdateCallbackEvent.DownloadProgressBody();
        event.union.download_progress.downloadProgress = new DownloadProgress();
        event.union.download_progress.downloadProgress.totalSize = 1024;
        event.union.download_progress.downloadProgress.downloaded = 512;
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0));
            return null;
        }).when(updateService).register(isA(UpdateCallback.class));
        when(updateService.getState()).thenReturn(UpdateState.DOWNLOADING);
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        controller.initialize(url, resourceBundle);

        var listener = listenerHolder.get();
        listener.callback(event);

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> controller.progressBar.getProgress() == 0.5);
    }
}