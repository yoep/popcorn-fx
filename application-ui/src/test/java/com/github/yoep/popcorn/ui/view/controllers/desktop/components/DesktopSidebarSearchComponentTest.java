package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.ui.events.RequestSearchFocus;
import com.github.yoep.popcorn.ui.view.controls.SearchTextField;
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

import static org.mockito.Mockito.spy;
import static org.mockito.Mockito.verify;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class DesktopSidebarSearchComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private DesktopSidebarSearchComponent component;

    @BeforeEach
    void setUp() {
        component.searchInput = spy(new SearchTextField());
    }

    @Test
    void testOnRequestSearchFocus() {
        component.initialize(url, resourceBundle);

        eventPublisher.publish(new RequestSearchFocus(this));
        WaitForAsyncUtils.waitForFxEvents();

        verify(component.searchInput).requestFocus();
    }
}