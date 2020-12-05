package com.github.yoep.popcorn.ui.view.controllers.tv.sections;

import com.github.yoep.popcorn.ui.view.controls.Overlay;
import com.github.yoep.popcorn.ui.view.controls.OverlayListener;
import javafx.scene.Node;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.net.MalformedURLException;
import java.net.URL;
import java.util.ResourceBundle;

import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class SettingsSectionControllerTest {
    @Mock
    private Overlay overlay;
    @Mock
    private ResourceBundle resourceBundle;

    private SettingsSectionController settingsSectionController;

    @BeforeEach
    void setUp() {
        settingsSectionController = new SettingsSectionController(overlay);
    }

    @Test
    void testSetBackspaceActionEnabled_whenInvoked_shouldDelegateToOverlay() {
        var expectedResult = false;

        settingsSectionController.setBackspaceActionEnabled(expectedResult);

        verify(overlay).setBackspaceActionEnabled(expectedResult);
    }

    @Test
    void testShowOverlay_whenInvoked_shouldDelegateToOverlay() {
        var originNode = mock(Node.class);
        var contents = mock(Node.class);

        settingsSectionController.showOverlay(originNode, contents);

        verify(overlay).show(originNode, contents);
    }

    @Nested
    class AddListenerTest {
        @Test
        void testAddListener_whenOverlayIsPresent_shouldDelegateToOverlay() {
            var listener = createListener();

            settingsSectionController.addListener(listener);

            verify(overlay).addListener(listener);
        }

        @Test
        void testAddListener_whenOverlayIsNotPresent_shouldAddListenerToBuffer() {
            var listener = createListener();
            var settingsSectionController = new SettingsSectionController();

            settingsSectionController.addListener(listener);

            verify(overlay, times(0)).addListener(listener);
        }

        @Test
        void testAddListener_whenInitializeIsInvoked_shouldUnloadListenersBuffer() throws MalformedURLException {
            var listener = createListener();
            var settingsSectionController = new SettingsSectionController();
            var url = new URL("http://www.lipsum.com");

            settingsSectionController.addListener(listener);
            settingsSectionController.setOverlay(overlay);
            settingsSectionController.initialize(url, resourceBundle);

            verify(overlay).addListener(listener);
        }
    }

    private OverlayListener createListener() {
        return () -> {
        };
    }
}
