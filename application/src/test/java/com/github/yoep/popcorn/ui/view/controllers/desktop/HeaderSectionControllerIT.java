package com.github.yoep.popcorn.ui.view.controllers.desktop;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.Categories;
import com.github.yoep.popcorn.TestFxBase;
import com.github.yoep.popcorn.matchers.StyleClassMatchers;
import javafx.scene.control.Labeled;
import javafx.scene.control.TextField;
import javafx.scene.input.MouseButton;
import org.junit.jupiter.api.Disabled;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;
import org.testfx.api.FxAssert;
import org.testfx.matcher.base.NodeMatchers;
import org.testfx.util.WaitForAsyncUtils;

import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static org.junit.jupiter.api.Assertions.assertTrue;

@Disabled
public class HeaderSectionControllerIT extends TestFxBase {
    @Nested
    class Icons {
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

    @Nested
    class SearchField {
        @Test
        void testSearch_whenSearchIsEntered_shouldShowExpectedSearchResult() throws TimeoutException {
            var movieCategory = lookup(Categories.MOVIES).queryLabeled();
            var searchField = lookup("#search .text-field").queryAs(TextField.class);
            var searchText = "zombieland";

            clickOn(movieCategory);

            // wait for poster items
            WaitForAsyncUtils.waitFor(20, TimeUnit.SECONDS, () -> lookup(".poster-item").queryAll().size() > 1);

            // search
            clickOn(searchField);
            write(searchText);

            sleep(500);

            // wait for the expected poster items
            WaitForAsyncUtils.waitFor(30, TimeUnit.SECONDS, () -> lookup(".poster-item").queryAll().size() >= 2);

            // check the labels of the poster items
            var titleLabels = lookup(".poster-item .title").queryAllAs(Labeled.class);

            assertTrue(titleLabels.stream().anyMatch(e -> e.getText().equals("Zombieland")));
            assertTrue(titleLabels.stream().anyMatch(e -> e.getText().equals("Zombieland: Double Tap")));
        }

        @Test
        void testSearch_whenSearchIsCleared_shouldShowNonFilteredCategoryItems() throws TimeoutException {
            var movieCategory = lookup(Categories.MOVIES).queryLabeled();
            var searchField = lookup("#search .text-field").queryAs(TextField.class);
            var searchClear = lookup("#search .clear-icon").queryAs(Icon.class);
            var searchText = "lorem";

            clickOn(movieCategory);

            // wait for poster items
            WaitForAsyncUtils.waitFor(20, TimeUnit.SECONDS, () -> lookup(".poster-item").queryAll().size() > 1);

            // search
            clickOn(searchField);
            write(searchText);

            // wait for the search field invoke
            sleep(1000);

            // wait for the expected poster items
            WaitForAsyncUtils.waitFor(30, TimeUnit.SECONDS, () -> lookup(".poster-item").queryAll().size() == 0);

            // click the search clear
            clickOn(searchClear);

            // wait for poster items
            WaitForAsyncUtils.waitFor(20, TimeUnit.SECONDS, () -> lookup(".poster-item").queryAll().size() > 1);
            assertTrue(lookup(".poster-item").queryAll().size() > 1);
        }
    }
}
