package com.github.yoep.popcorn.ui.view.controllers;

import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyCodeCombination;
import javafx.scene.input.KeyCombination;
import javafx.scene.input.KeyEvent;
import lombok.NoArgsConstructor;

@NoArgsConstructor
public final class KeyboardShortcuts {
    static final KeyCodeCombination PASTE = new KeyCodeCombination(KeyCode.V, KeyCombination.SHORTCUT_DOWN);

    static final KeyCodeCombination UI_ENLARGE_1 = new KeyCodeCombination(KeyCode.ADD, KeyCombination.SHORTCUT_DOWN);
    static final KeyCodeCombination UI_ENLARGE_2 = new KeyCodeCombination(KeyCode.PLUS, KeyCombination.SHORTCUT_DOWN);
    static final KeyCodeCombination UI_ENLARGE_3 = new KeyCodeCombination(KeyCode.EQUALS, KeyCombination.SHORTCUT_DOWN, KeyCombination.SHIFT_DOWN);

    static final KeyCodeCombination UI_REDUCE_1 = new KeyCodeCombination(KeyCode.SUBTRACT, KeyCombination.SHORTCUT_DOWN);
    static final KeyCodeCombination UI_REDUCE_2 = new KeyCodeCombination(KeyCode.MINUS, KeyCombination.SHORTCUT_DOWN);

    static final KeyCodeCombination UI_RESET = new KeyCodeCombination(KeyCode.EQUALS, KeyCombination.SHORTCUT_DOWN);

    static boolean isPaste(KeyEvent e) {
        return PASTE.match(e);
    }

    static boolean isUiEnlarge(KeyEvent e) {
        return UI_ENLARGE_1.match(e) || UI_ENLARGE_2.match(e) || UI_ENLARGE_3.match(e);
    }

    static boolean isUiReduce(KeyEvent e) {
        return UI_REDUCE_1.match(e) || UI_REDUCE_2.match(e);
    }

    static boolean isUiReset(KeyEvent e) {
        return UI_RESET.match(e);
    }
}
