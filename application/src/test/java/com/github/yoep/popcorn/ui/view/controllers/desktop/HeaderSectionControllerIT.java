package com.github.yoep.popcorn.ui.view.controllers.desktop;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.TestFxBase;
import javafx.scene.input.MouseButton;
import org.junit.jupiter.api.Test;
import org.testfx.api.FxAssert;
import org.testfx.matcher.base.NodeMatchers;

import static org.junit.jupiter.api.Assertions.assertTrue;

public class HeaderSectionControllerIT extends TestFxBase {
    @Test
    void testIcons_whenSettingsIsClicked_shouldShowSettings() {
        var settingsIcon = (Icon) lookup("#settingsIcon").query();

        clickOn(settingsIcon, MouseButton.PRIMARY);

        FxAssert.verifyThat("#settings", NodeMatchers.isVisible());
        assertTrue(settingsIcon.getStyleClass().contains("active"));
    }
}
