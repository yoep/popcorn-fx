package com.github.yoep.popcorn.ui.view.controllers.tv;

import com.github.yoep.popcorn.Categories;
import javafx.scene.Node;
import javafx.scene.input.KeyCode;
import org.apache.commons.lang3.StringUtils;
import org.junit.jupiter.api.Test;
import org.testfx.api.FxAssert;
import org.testfx.util.WaitForAsyncUtils;

import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static org.junit.jupiter.api.Assertions.assertTrue;

public class ListSectionControllerIT extends AbstractTvSectionControllerIT {

    @Test
    void testInfiniteScrollPane_whenDetailsAreClosed_shouldFocusLastKnownFocusedItem() throws TimeoutException {
        setUp();

        var movieCategory = lookup(Categories.MOVIES).queryAs(Node.class);

        clickOn(movieCategory);

        // wait for poster items
        WaitForAsyncUtils.waitFor(20, TimeUnit.SECONDS, () -> lookup(".poster-item").queryAll().size() > 1);

        // wait till the first item has focus
        WaitForAsyncUtils.waitFor(10, TimeUnit.SECONDS, () -> lookup(".poster-item").queryAs(Node.class).isFocused());

        // focus an item lower in the list
        for (int i = 0; i < 5; i++) {
            press(KeyCode.DOWN);
            release(KeyCode.DOWN);
        }

        // remember the current focused item
        var focusedNode = lookup(".poster-item").queryAllAs(Node.class).stream()
                .filter(Node::isFocusTraversable)
                .filter(Node::isFocused)
                .findFirst();

        // check if the node is found
        assertTrue(focusedNode.isPresent(), "Focused node could not be found");

        // go to the details of the current item
        press(KeyCode.ENTER);
        release(KeyCode.ENTER);

        WaitForAsyncUtils.waitFor(20, TimeUnit.SECONDS, () -> StringUtils.isNotEmpty(lookup("#title").queryLabeled().getText()));

        // go back to the list
        clickOn(lookup("#closeButton").queryAs(Node.class));

        WaitForAsyncUtils.waitFor(20, TimeUnit.SECONDS, () -> lookup(".poster-item").queryAll().size() > 1);

        // check if the item is focused again
        FxAssert.verifyThat(focusedNode.get(), Node::isFocused);
    }
}
