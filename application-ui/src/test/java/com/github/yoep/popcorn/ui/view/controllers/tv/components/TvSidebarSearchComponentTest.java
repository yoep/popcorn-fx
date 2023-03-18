package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.ui.events.RequestSearchFocus;
import javafx.scene.input.MouseEvent;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class TvSidebarSearchComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @InjectMocks
    private TvSidebarSearchComponent component;

    @Test
    void testOnSearchClicked() {
        var event = mock(MouseEvent.class);

        component.onSearchClicked(event);

        verify(event).consume();
        verify(eventPublisher).publish(new RequestSearchFocus(component));
    }
}