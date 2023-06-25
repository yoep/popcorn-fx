package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.InfoNotificationEvent;
import javafx.scene.Scene;
import javafx.scene.layout.BorderPane;
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

import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.when;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class NotificationSectionControllerTest {
    @Mock
    private ViewLoader viewLoader;
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private NotificationSectionController controller;

    @BeforeEach
    void setUp() {
        controller.rootPane = new Pane();
    }

    @Test
    void testOnNotificationEvent() throws TimeoutException {
        var node = new BorderPane();
        when(viewLoader.load(eq(NotificationSectionController.NOTIFICATION_VIEW), isA(Object.class))).thenReturn(node);
        controller.init();

        eventPublisher.publish(new InfoNotificationEvent(this, "lorem"));
        var scene = new Scene(controller.rootPane);
        WaitForAsyncUtils.waitForFxEvents();
        controller.initialize(url, resourceBundle);

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> controller.rootPane.getChildren().contains(node));
    }
}