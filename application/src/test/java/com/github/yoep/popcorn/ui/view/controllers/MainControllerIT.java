package com.github.yoep.popcorn.ui.view.controllers;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.PopcornTimeApplicationTest;
import com.github.yoep.popcorn.TestFxBase;
import javafx.stage.Stage;
import org.junit.jupiter.api.Test;
import org.testfx.api.FxAssert;
import org.testfx.api.FxToolkit;
import org.testfx.matcher.base.NodeMatchers;
import org.testfx.util.WaitForAsyncUtils;

import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

public class MainControllerIT extends TestFxBase {
    static final String CONTENT_PANE_ID = "#contentPane";
    static final String PLAYER_PANE_ID = "#playerPane";

    @Override
    public void setUp() {
        //no-op
    }

    @Override
    public void init() throws Exception {
        FxToolkit.registerStage(Stage::new);
    }

    @Override
    public void stop() throws Exception {
        FxToolkit.hideStage();
        application.stop();
    }

    @Test
    public void testStartup_whenNoNonOptionArgumentAreGiven_shouldShowContentAsStartScreen() throws TimeoutException {
        application = (PopcornTimeApplicationTest) FxToolkit.setupApplication(PopcornTimeApplicationTest.class);
        FxToolkit.showStage();
        WaitForAsyncUtils.waitForFxEvents(100);

        FxAssert.verifyThat(CONTENT_PANE_ID, NodeMatchers.isNotNull());
    }

    @Test
    public void testStartup_whenNonOptionArgumentIsGiven_shouldShowPlayerAsStartScreen() throws TimeoutException {
        application = (PopcornTimeApplicationTest) FxToolkit.setupApplication(PopcornTimeApplicationTest.class, "https://www.youtube.com/watch?v=BSF5yoD-vC4");
        FxToolkit.showStage();
        WaitForAsyncUtils.waitForFxEvents(100);

        FxAssert.verifyThat(PLAYER_PANE_ID, NodeMatchers.isNotNull());

        // close the video player
        var closePlayerIcon = lookup("#closePlayer").queryAs(Icon.class);

        clickOn(closePlayerIcon);

        WaitForAsyncUtils.waitFor(20, TimeUnit.SECONDS, () -> lookup(CONTENT_PANE_ID).tryQuery().isPresent());
    }
}
