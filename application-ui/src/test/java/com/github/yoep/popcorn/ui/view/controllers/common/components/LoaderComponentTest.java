package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.loader.LoaderListener;
import com.github.yoep.popcorn.backend.loader.LoaderService;
import com.github.yoep.popcorn.ui.events.CloseLoadEvent;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.scene.control.Label;
import javafx.scene.control.ProgressBar;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
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
import java.util.concurrent.atomic.AtomicReference;

import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class LoaderComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private LoaderService service;
    @Mock
    private LocaleText localeText;
    @Mock
    private ImageService imageService;
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private LoaderComponent component;

    private final AtomicReference<LoaderListener> listenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0));
            return null;
        }).when(service).addListener(isA(LoaderListener.class));

        component.infoPane = new Pane(new Label(), new Label(), new Label());
        component.loaderActions = new Pane();
        component.progressBar = new ProgressBar();
        component.statusText = new Label();
    }

    @Test
    void testOnCancelClicked() {
        var event = mock(MouseEvent.class);
        var progressPane = new Pane();
        when(viewLoader.load(LoaderComponent.PROGRESS_INFO_VIEW, component.infoComponent)).thenReturn(progressPane);
        component.initialize(url, resourceBundle);

        component.onCancelClicked(event);

        verify(event).consume();
        verify(service).cancel();
        verify(eventPublisher).publish(new CloseLoadEvent(component));
    }

    @Test
    void testOnCancelKeyPressed() {
        var event = mock(KeyEvent.class);
        var progressPane = new Pane();
        when(viewLoader.load(LoaderComponent.PROGRESS_INFO_VIEW, component.infoComponent)).thenReturn(progressPane);
        when(event.getCode()).thenReturn(KeyCode.ENTER);
        component.initialize(url, resourceBundle);

        component.onCancelPressed(event);

        verify(event).consume();
        verify(service).cancel();
        verify(eventPublisher).publish(new CloseLoadEvent(component));
    }

    @Test
    void testOnLoaderKeyPressed() {
        var backEvent = mock(KeyEvent.class);
        when(backEvent.getCode()).thenReturn(KeyCode.BACK_SPACE);
        var escEvent = mock(KeyEvent.class);
        when(escEvent.getCode()).thenReturn(KeyCode.BACK_SPACE);
        var progressPane = new Pane();
        when(viewLoader.load(LoaderComponent.PROGRESS_INFO_VIEW, component.infoComponent)).thenReturn(progressPane);
        component.initialize(url, resourceBundle);

        component.onLoaderKeyPressed(backEvent);
        verify(backEvent).consume();
        verify(service).cancel();
        reset(service);

        component.onLoaderKeyPressed(escEvent);
        verify(escEvent).consume();
        verify(service).cancel();
    }
}