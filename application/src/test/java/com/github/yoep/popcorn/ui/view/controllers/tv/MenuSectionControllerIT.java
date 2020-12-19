package com.github.yoep.popcorn.ui.view.controllers.tv;

import com.github.yoep.popcorn.Categories;
import com.github.yoep.popcorn.PopcornTimeApplicationTest;
import com.github.yoep.popcorn.TestFxBase;
import javafx.scene.Node;
import javafx.scene.control.TextField;
import javafx.stage.Stage;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;
import org.testfx.api.FxToolkit;
import org.testfx.util.WaitForAsyncUtils;

import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static org.junit.jupiter.api.Assertions.assertTrue;

public class MenuSectionControllerIT extends TestFxBase {
    @Override
    public void setUp() throws TimeoutException {
        application = (PopcornTimeApplicationTest) FxToolkit.setupApplication(PopcornTimeApplicationTest.class, "--tv");
        FxToolkit.showStage();
        WaitForAsyncUtils.waitForFxEvents(100);
    }

    @Override
    public void init() throws Exception {
        FxToolkit.registerStage(Stage::new);
    }

    @Override
    public void stop() throws Exception {
        FxToolkit.hideStage();
    }

    @Nested
    class SearchField {
        @Test
        void testSearch_shouldClearSearchFilter_whenCategoryIsChanged() throws TimeoutException {
            setUp();

            var movieCategory = lookup(Categories.MOVIES).queryAs(Node.class);
            var seriesCategory = lookup(Categories.SERIES).queryAs(Node.class);
            var searchField = lookup("#searchField").queryAs(TextField.class);
            var searchText = "zombieland";

            clickOn(movieCategory);

            // wait for poster items
            WaitForAsyncUtils.waitFor(20, TimeUnit.SECONDS, () -> lookup(".poster-item").queryAll().size() > 1);

            // search
            clickOn(searchField);
            write(searchText);

            sleep(4000);

            // wait for the expected poster items
            WaitForAsyncUtils.waitFor(30, TimeUnit.SECONDS, () -> lookup(".poster-item").queryAll().size() >= 2);

            var totalItems = lookup(".poster-item").queryAll().size();

            // switch to another category
            clickOn(seriesCategory);

            // check if the search field is cleared
            WaitForAsyncUtils.waitFor(5, TimeUnit.SECONDS, () -> searchField.getText().equals(""));

            // wait for poster items
            WaitForAsyncUtils.waitFor(20, TimeUnit.SECONDS, () -> lookup(".poster-item").queryAll().size() > 1);

            // switch back to movies
            clickOn(movieCategory);

            // wait for poster items
            WaitForAsyncUtils.waitFor(20, TimeUnit.SECONDS, () -> lookup(".poster-item").queryAll().size() > 1);

            assertTrue(lookup(".poster-item").queryAll().size() != totalItems);
        }
    }
}
