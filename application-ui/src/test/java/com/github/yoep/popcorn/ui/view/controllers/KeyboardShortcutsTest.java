package com.github.yoep.popcorn.ui.view.controllers;

import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.testfx.framework.junit5.ApplicationExtension;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

@ExtendWith(ApplicationExtension.class)
class KeyboardShortcutsTest {
    @Test
    void testIsPaste() {
        var event1 = new KeyEvent(
                this,
                null,
                KeyEvent.KEY_PRESSED,
                KeyCode.V.getChar(),
                KeyCode.V.getName(),
                KeyCode.V,
                false,
                !isMacOs(),
                false,
                isMacOs());
        assertTrue(KeyboardShortcuts.isPaste(event1));

        var event2 = new KeyEvent(
                this,
                null,
                KeyEvent.KEY_PRESSED,
                KeyCode.V.getChar(),
                KeyCode.V.getName(),
                KeyCode.V,
                false,
                false,
                false,
                false);
        assertFalse(KeyboardShortcuts.isPaste(event2));
    }

    @Test
    void testIsUiEnlarge() {
        var event1 = new KeyEvent(
                this,
                null,
                KeyEvent.KEY_PRESSED,
                KeyCode.PLUS.getChar(),
                KeyCode.PLUS.getName(),
                KeyCode.PLUS,
                false,
                !isMacOs(),
                false,
                isMacOs());
        assertTrue(KeyboardShortcuts.isUiEnlarge(event1));

        var event2 = new KeyEvent(
                this,
                null,
                KeyEvent.KEY_PRESSED,
                KeyCode.ADD.getChar(),
                KeyCode.ADD.getName(),
                KeyCode.ADD,
                false,
                !isMacOs(),
                false,
                isMacOs());
        assertTrue(KeyboardShortcuts.isUiEnlarge(event2));
    }

    @Test
    void testIsUiReduce() {
        var event1 = new KeyEvent(
                this,
                null,
                KeyEvent.KEY_PRESSED,
                KeyCode.MINUS.getChar(),
                KeyCode.MINUS.getName(),
                KeyCode.MINUS,
                false,
                !isMacOs(),
                false,
                isMacOs());
        assertTrue(KeyboardShortcuts.isUiReduce(event1));

        var event2 = new KeyEvent(
                this,
                null,
                KeyEvent.KEY_PRESSED,
                KeyCode.SUBTRACT.getChar(),
                KeyCode.SUBTRACT.getName(),
                KeyCode.SUBTRACT,
                false,
                !isMacOs(),
                false,
                isMacOs());
        assertTrue(KeyboardShortcuts.isUiReduce(event2));
    }

    @Test
    void testIsUiReset() {
        var event = new KeyEvent(
                this,
                null,
                KeyEvent.KEY_PRESSED,
                KeyCode.EQUALS.getChar(),
                KeyCode.EQUALS.getName(),
                KeyCode.EQUALS,
                false,
                !isMacOs(),
                false,
                isMacOs());
        assertTrue(KeyboardShortcuts.isUiReset(event));
    }

    private static boolean isMacOs() {
        return System.getProperty("os.name").toLowerCase().contains("mac");
    }
}