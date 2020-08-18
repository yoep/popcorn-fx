package com.github.yoep.popcorn.ui.view.controllers.desktop;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.TestFxBase;
import com.github.yoep.popcorn.matchers.StyleClassMatchers;
import javafx.scene.input.MouseButton;
import org.junit.jupiter.api.Test;
import org.testfx.api.FxAssert;
import org.testfx.matcher.base.NodeMatchers;

public class HeaderSectionControllerIT extends TestFxBase {
    @Test
    void testIcons_whenSettingsIsClicked_shouldShowSettings() {
        var settingsIcon = lookup("#settingsIcon").queryAs(Icon.class);

        clickOn(settingsIcon, MouseButton.PRIMARY);

        FxAssert.verifyThat("#settings", NodeMatchers.isVisible());
        FxAssert.verifyThat(settingsIcon, StyleClassMatchers.hasStyleClass("active"));
    }

    @Test
    void testIcons_whenTorrentCollectionIsClicked_shouldShowTorrentCollection() {
        var torrentCollectionIcon = lookup("#torrentCollectionIcon").queryAs(Icon.class);

        clickOn(torrentCollectionIcon, MouseButton.PRIMARY);

        FxAssert.verifyThat("#torrentCollection", NodeMatchers.isVisible());
        FxAssert.verifyThat(torrentCollectionIcon, StyleClassMatchers.hasStyleClass("active"));
    }
}
