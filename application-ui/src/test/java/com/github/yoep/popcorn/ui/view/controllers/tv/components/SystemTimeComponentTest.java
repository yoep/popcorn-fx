package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import javafx.scene.control.Label;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.net.URL;
import java.util.ResourceBundle;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class SystemTimeComponentTest {
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private SystemTimeComponent component;

    @BeforeEach
    void setUp() {
        component.time = new Label();
    }

    @Test
    void testInitialize_shouldUpdateTime() {
        component.initialize(url, resourceBundle);

        assertNotNull(component.time.getText());
        assertFalse(component.time.getText().isEmpty());
    }
}