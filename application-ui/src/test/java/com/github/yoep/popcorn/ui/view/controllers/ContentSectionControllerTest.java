package com.github.yoep.popcorn.ui.view.controllers;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.settings.OptionsService;
import com.github.yoep.popcorn.ui.events.*;
import com.github.yoep.popcorn.ui.view.services.MaximizeService;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseButton;
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
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class ContentSectionControllerTest {
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private LocaleText localeText;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private MaximizeService maximizeService;
    @Mock
    private OptionsService optionsService;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private ContentSectionController controller;

    @BeforeEach
    void setUp() {
        lenient().when(viewLoader.load(isA(String.class))).thenReturn(new Pane());
        controller.contentPane = new Pane();
        controller.listPane = new Pane();

        controller.contentPane.getChildren().add(controller.listPane);
        controller.contentPane.getChildren().add(new Pane());
        controller.contentPane.setOnMouseClicked(controller::onMouseClicked);
    }

    @Test
    void testOnCategoryChangedEvent() {
        controller.initialize(url, resourceBundle);

        eventPublisher.publish(new CategoryChangedEvent(controller, Category.SERIES));

        assertEquals(ContentSectionController.ContentType.LIST, controller.activeType);
        assertEquals(controller.listPane, controller.contentPane.getChildren().get(0));
    }

    @Test
    void testOnShowSettingsEvent() throws TimeoutException {
        var settingsPane = new Pane();
        when(viewLoader.load(ContentSectionController.SETTINGS_SECTION)).thenReturn(settingsPane);

        controller.initialize(url, resourceBundle);
        verify(viewLoader, timeout(250)).load(ContentSectionController.SETTINGS_SECTION);

        eventPublisher.publish(new ShowSettingsEvent(controller));
        WaitForAsyncUtils.waitForFxEvents();

        assertEquals(ContentSectionController.ContentType.SETTINGS, controller.activeType);
        WaitForAsyncUtils.waitFor(500, TimeUnit.MILLISECONDS, () -> controller.contentPane.getChildren().contains(settingsPane));
    }

    @Test
    void testMouseDoubleClicked_shouldToggleTheMaximizeState() {
        var event = mock(MouseEvent.class);
        when(event.getSceneY()).thenReturn(30.0);
        when(event.getButton()).thenReturn(MouseButton.PRIMARY);
        when(event.getClickCount()).thenReturn(2);
        controller.initialize(url, resourceBundle);

        controller.onMouseClicked(event);

        verify(maximizeService).setMaximized(true);
    }

    @Test
    void testWhenDesktop_shouldLoadWindowComponent() {
        when(optionsService.isTvMode()).thenReturn(false);

        controller.initialize(url, resourceBundle);

        verify(viewLoader).load(ContentSectionController.WINDOW_COMPONENT);
    }

    @Test
    void testWhenTv_shouldLoadSystemTimeComponent() {
        when(optionsService.isTvMode()).thenReturn(true);

        controller.initialize(url, resourceBundle);

        verify(viewLoader).load(ContentSectionController.SYSTEM_TIME_COMPONENT);
    }

    @Test
    void testOnCloseAbout() {
        controller.initialize(url, resourceBundle);

        eventPublisher.publish(new ShowAboutEvent(this));
        WaitForAsyncUtils.waitForFxEvents();
        assertEquals(ContentSectionController.ContentType.ABOUT, controller.activeType);

        eventPublisher.publish(new CloseAboutEvent(this));
        WaitForAsyncUtils.waitForFxEvents();
        assertEquals(ContentSectionController.ContentType.LIST, controller.activeType);
    }

    @Test
    void testOnKeyPressed_whenHomeIsPressed() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.HOME);
        controller.initialize(url, resourceBundle);

        controller.onKeyPressed(event);

        verify(event).consume();
        verify(eventPublisher).publish(new HomeEvent(controller));
    }

    @Test
    void testOnKeyPressed_whenContextMenuIsPressed() {
        var event = mock(KeyEvent.class);
        when(event.getCode()).thenReturn(KeyCode.CONTEXT_MENU);
        controller.initialize(url, resourceBundle);

        controller.onKeyPressed(event);

        verify(event).consume();
        verify(eventPublisher).publish(new ContextMenuEvent(controller));
    }
}