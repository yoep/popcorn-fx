package com.github.yoep.popcorn.ui.font;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertNotNull;

class PopcornFontRegistryTest {
    @Test
    void testGetInstance() {
        var result = PopcornFontRegistry.getInstance();

        assertNotNull(result, "expected a font registry instance to have been returned");
    }

    @Test
    void testLoadFont() {
        var registry = PopcornFontRegistry.getInstance();

        var font = registry.loadFont("fontawesome-regular.ttf");

        assertNotNull(font, "expected a font to have been returned");
    }
}