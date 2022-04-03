package com.github.yoep.popcorn.ui.view.controllers.desktop;

import com.github.yoep.popcorn.Categories;
import com.github.yoep.popcorn.TestFxBase;
import com.github.yoep.popcorn.backend.adapters.torrent.TorrentService;
import com.github.yoep.popcorn.backend.adapters.torrent.state.SessionState;
import javafx.scene.Node;
import javafx.scene.layout.GridPane;
import org.apache.commons.lang3.StringUtils;
import org.junit.jupiter.api.Disabled;
import org.junit.jupiter.api.Test;
import org.testfx.api.FxAssert;
import org.testfx.util.WaitForAsyncUtils;

import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

@Disabled
public class MovieDetailsComponentIT extends TestFxBase {
    @Test
    void testMovieDetails_whenMovieIsClick_shouldShowDetails() throws TimeoutException {
        var movieCategory = lookup(Categories.MOVIES).queryLabeled();

        clickOn(movieCategory);

        // wait for torrent session
        WaitForAsyncUtils.waitFor(20, TimeUnit.SECONDS, this::waitForTorrentSession);
        // wait for poster items
        WaitForAsyncUtils.waitFor(20, TimeUnit.SECONDS, () -> lookup(".poster-item").queryAll().size() > 1);

        var posters = lookup(".poster-item").queryAllAs(GridPane.class);

        clickOn(posters.stream().findFirst().orElse(null));

        WaitForAsyncUtils.waitFor(20, TimeUnit.SECONDS, this::isTitleAvailable);
        FxAssert.verifyThat(".movie-details", Node::isVisible);
        FxAssert.verifyThat("#title", Node::isVisible);
        FxAssert.verifyThat(lookup("#year").queryLabeled(), node -> StringUtils.isNotEmpty(node.getText()));
        FxAssert.verifyThat(lookup("#duration").queryLabeled(), node -> StringUtils.isNotEmpty(node.getText()));
        FxAssert.verifyThat(lookup("#genres").queryLabeled(), node -> StringUtils.isNotEmpty(node.getText()));
        FxAssert.verifyThat(lookup("#overview").queryLabeled(), node -> StringUtils.isNotEmpty(node.getText()));
    }

    private boolean isTitleAvailable() {
        var title = lookup("#title").queryLabeled();

        return StringUtils.isNotEmpty(title.getText());
    }

    private boolean waitForTorrentSession() {
        var applicationContext = application.getContext();
        var torrentService = applicationContext.getBean(TorrentService.class);

        return torrentService.getSessionState() == SessionState.RUNNING;
    }
}
