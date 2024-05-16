package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.ui.view.services.MaximizeService;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.scene.image.ImageView;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.ResourceBundle;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.when;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class WindowComponentTest {
    @Mock
    private MaximizeService maximizeService;
    @Mock
    private PlatformProvider platformProvider;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;

    private WindowComponent component;

    private BooleanProperty maximizedProperty = new SimpleBooleanProperty();

    @BeforeEach
    void setUp() {
        when(maximizeService.maximizedProperty()).thenReturn(maximizedProperty);

        component = new WindowComponent(maximizeService, platformProvider);
        component.maximizeImageView = new ImageView();
    }

    @Test
    void testInitialize() {
        when(maximizeService.isMaximized()).thenReturn(true);
        component.initialize(url, resourceBundle);

        var result = component.maximizeImageView.getImage();

        assertEquals(component.restoreImage, result);
    }

    @Test
    void testMaximizedPropertyChanged() {
        component.initialize(url, resourceBundle);

        maximizedProperty.set(true);
        WaitForAsyncUtils.waitForFxEvents();
        assertEquals(component.restoreImage, component.maximizeImageView.getImage(), "Restore image should be shown when maximized property is set to true");

        maximizedProperty.set(false);
        WaitForAsyncUtils.waitForFxEvents();
        assertEquals(component.maximizeImage, component.maximizeImageView.getImage(), "Maximize image should be shown when maximized property is set to false");
    }
}