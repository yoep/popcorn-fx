package com.github.yoep.player.popcorn.controllers.components;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.PlayVideoEvent;
import javafx.scene.control.Label;
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
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class PlayerHeaderComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private PlayerHeaderComponent component;

    @BeforeEach
    void setUp() {
        component.playerHeader = new GridPane();
        component.title = new Label();
    }

    @Test
    void testInitializeMode() throws TimeoutException {
        var actions = new Pane();
        when(viewLoader.load(isA(String.class))).thenReturn(actions);

        component.initialize(url, resourceBundle);

        verify(viewLoader).load(PlayerHeaderComponent.VIEW_HEADER_ACTIONS);
        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> component.playerHeader.getChildren().contains(actions));
    }

    @Test
    void testOnPlayVideoEvent() throws TimeoutException {
        var title = "lorem ipsum";
        when(viewLoader.load(PlayerHeaderComponent.VIEW_HEADER_ACTIONS)).thenReturn(new Pane());
        component.initialize(url, resourceBundle);

        eventPublisher.publish(new PlayVideoEvent(this, "http://localhost", title, false));

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> component.title.getText().equals(title));
    }
}