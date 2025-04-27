package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Update;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.UpdateEvent;
import com.github.yoep.popcorn.backend.updater.UpdateEventListener;
import com.github.yoep.popcorn.backend.updater.UpdateService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
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
    private UpdateSectionController controller;

    private final AtomicReference<UpdateEventListener> eventListenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocations -> {
            eventListenerHolder.set(invocations.getArgument(0, UpdateEventListener.class));
            return null;
        }).when(updateService).addListener(isA(UpdateEventListener.class));

        controller = new UpdateSectionController(updateService, imageService, localeText, eventPublisher);

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
        when(updateService.getState()).thenReturn(CompletableFuture.completedFuture(Update.State.UPDATE_AVAILABLE));
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
        when(updateService.getState()).thenReturn(CompletableFuture.completedFuture(Update.State.DOWNLOADING));
        when(imageService.loadResource(isA(String.class))).thenReturn(new CompletableFuture<>());
        controller.initialize(url, resourceBundle);

        var listener = eventListenerHolder.get();
        listener.onDownloadProgress(UpdateEvent.DownloadProgress.newBuilder()
                .setProgress(Update.DownloadProgress.newBuilder()
                        .setDownloaded(512)
                        .setTotalSize(1024)
                        .build())
                .build());

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> controller.progressBar.getProgress() == 0.5);
    }
}