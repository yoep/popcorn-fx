package com.github.yoep.popcorn.ui.view.controls;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.testfx.framework.junit5.ApplicationExtension;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

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
    }
}