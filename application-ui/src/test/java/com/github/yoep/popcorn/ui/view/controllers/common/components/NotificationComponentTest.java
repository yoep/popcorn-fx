package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.events.InfoNotificationEvent;
import javafx.scene.control.Label;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicBoolean;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class NotificationComponentTest {
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;

    @Test
    void testInitialize() {
        var expectedText = "Lorem ipsum dolor";
        var notificationEvent = new InfoNotificationEvent(this, expectedText);
        var component = initializeFields(new NotificationComponent(notificationEvent));

        component.initialize(url, resourceBundle);

        assertEquals(expectedText, component.text.getText());
    }

    @Test
    void testOnClicked() {
        var event = mock(MouseEvent.class);
        var actionResult = new AtomicBoolean(false);
        var closeResult = new AtomicBoolean(false);
        var notificationEvent = new InfoNotificationEvent(this, "lorem", () -> actionResult.set(true));
        var component = initializeFields(new NotificationComponent(notificationEvent));
        component.setOnClose(closeEvent -> closeResult.set(true));

        component.onClicked(event);

        verify(event).consume();
        assertTrue(actionResult.get(), "expected the action to have been handled");
        assertTrue(closeResult.get(), "expected the onClose action to have been handled");
    }

    private static NotificationComponent initializeFields(NotificationComponent component) {
        component.rootPane = new Pane();
        component.text = new Label();
        return component;
    }
}