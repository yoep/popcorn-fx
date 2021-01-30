package com.github.yoep.popcorn.ui.view.controllers.tv;

import com.github.yoep.popcorn.Categories;
import javafx.scene.Node;
import javafx.scene.control.TextField;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;
import org.testfx.util.WaitForAsyncUtils;

import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static org.junit.jupiter.api.Assertions.assertTrue;

public class MenuSectionControllerIT extends AbstractTvSectionControllerIT {

    @Nested
    class SearchField {
        @Test
        void testSearch_shouldClearSearchFilter_whenCategoryIsChanged() throws TimeoutException {
            setUp();

            var movieCategory = lookup(Categories.MOVIES).queryAs(Node.class);
            var seriesCategory = lookup(Categories.SERIES).queryAs(Node.class);
            var searchField = lookup("#searchField").queryAs(TextField.class);
            var searchText = "zombie";

            clickOn(movieCategory);

            // wait for poster items
            WaitForAsyncUtils.waitFor(20, TimeUnit.SECONDS, () -> lookup(".poster-item").queryAll().size() > 1);

            // wait till the first item is focused as it will break the write function
            WaitForAsyncUtils.waitFor(10, TimeUnit.SECONDS, () -> lookup(".poster-item").queryAs(Node.class).isFocused());

            // search
            clickOn(searchField);
            write(searchText, 10);

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
