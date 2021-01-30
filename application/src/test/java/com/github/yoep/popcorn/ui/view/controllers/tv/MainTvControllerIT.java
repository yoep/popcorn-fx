package com.github.yoep.popcorn.ui.view.controllers.tv;

import com.github.yoep.popcorn.PopcornTimeApplicationTest;
import com.github.yoep.popcorn.TestFxBase;
import com.github.yoep.popcorn.matchers.StylesheetMatcher;
import javafx.scene.layout.Pane;
import javafx.stage.Stage;
import org.junit.jupiter.api.Test;
import org.testfx.api.FxAssert;
import org.testfx.api.FxToolkit;
import org.testfx.util.WaitForAsyncUtils;

import java.util.concurrent.TimeoutException;

public class MainTvControllerIT extends TestFxBase {
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
    }

    @Test
    public void testStartup_whenTVModeIsPassed_shouldOpenTVMode() throws TimeoutException {
        application = (PopcornTimeApplicationTest) FxToolkit.setupApplication(PopcornTimeApplicationTest.class, "--tv", "--disable-popcorn-keys");
        FxToolkit.showStage();
        WaitForAsyncUtils.waitForFxEvents(100);

        FxAssert.verifyThat(lookup("#rootPane").queryAs(Pane.class), StylesheetMatcher.hasStyleSheet(MainTvController.TV_STYLESHEET));
    }
}
