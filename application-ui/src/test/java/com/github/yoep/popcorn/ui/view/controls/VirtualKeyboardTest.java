package com.github.yoep.popcorn.ui.view.controls;

import javafx.scene.input.MouseEvent;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.util.concurrent.atomic.AtomicBoolean;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.mock;

@ExtendWith(ApplicationExtension.class)
class VirtualKeyboardTest {
    @Test
    void testSetText() {
        var text = "test";
        var keyboard = new VirtualKeyboard();

        keyboard.setText(text);

        assertEquals(text, keyboard.getText());
    }

    @Test
    void testCreateDefaultSkin() {
        var keyboard = new VirtualKeyboard();

        var result = keyboard.createDefaultSkin();

        assertTrue(result instanceof VirtualKeyboard.VirtualKeyboardSkin);
        assertEquals(38, ((VirtualKeyboard.VirtualKeyboardSkin) result).grid.getChildren().size());
    }

    @Test
    void testEnableSpecialKeys() {
        var keyboard = new VirtualKeyboard();
        keyboard.setEnableSpecialKeys(true);

        var result = (VirtualKeyboard.VirtualKeyboardSkin) keyboard.createDefaultSkin();

        assertEquals(43, result.grid.getChildren().size());
    }

    @Test
    void testEnableCloseKey() {
        var event = mock(MouseEvent.class);
        var callback = new AtomicBoolean(false);
        var keyboard = new VirtualKeyboard();
        keyboard.setEnableCloseKey(true);
        keyboard.setOnClose(e -> callback.set(true));

        var result = (VirtualKeyboard.VirtualKeyboardSkin) keyboard.createDefaultSkin();
        assertEquals(39, result.grid.getChildren().size());

        result.closeButton.getOnMouseClicked().handle(event);
        WaitForAsyncUtils.waitForFxEvents();
        assertTrue(callback.get(), "expected the onClose action to have been invoked");
    }
}