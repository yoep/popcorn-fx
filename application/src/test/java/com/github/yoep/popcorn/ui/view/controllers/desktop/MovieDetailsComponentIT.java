package com.github.yoep.popcorn.ui.view.controllers.desktop;

import com.github.yoep.popcorn.TestFxBase;
import javafx.scene.Node;
import javafx.scene.control.Label;
import javafx.scene.layout.GridPane;
import org.apache.commons.lang3.StringUtils;
import org.junit.jupiter.api.Test;
import org.testfx.api.FxAssert;
import org.testfx.util.WaitForAsyncUtils;

import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

public class MovieDetailsComponentIT extends TestFxBase {
    @Test
    void testMovieDetails_whenMovieIsClick_shouldShowDetails() throws TimeoutException {
        var movieCategory = lookup("#moviesCategory").queryLabeled();

        clickOn(movieCategory);
        WaitForAsyncUtils.waitFor(10, TimeUnit.SECONDS, () -> lookup(".poster-item").queryAll().size() > 1);

        var posters = lookup(".poster-item").queryAllAs(GridPane.class);

        clickOn(posters.stream().findFirst().orElse(null));

        FxAssert.verifyThat(".movie-details", Node::isVisible);
        FxAssert.verifyThat("#title", node -> StringUtils.isNotEmpty(((Label) node).getText()));
    }
}
