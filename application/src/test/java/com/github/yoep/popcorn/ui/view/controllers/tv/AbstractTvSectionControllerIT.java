package com.github.yoep.popcorn.ui.view.controllers.tv;

import com.github.yoep.popcorn.PopcornTimeApplicationTest;
import com.github.yoep.popcorn.TestFxBase;
import javafx.stage.Stage;
import org.testfx.api.FxToolkit;
import org.testfx.util.WaitForAsyncUtils;

import java.util.concurrent.TimeoutException;

public abstract class AbstractTvSectionControllerIT extends TestFxBase {
    @Override
    public void setUp() throws TimeoutException {
        application = (PopcornTimeApplicationTest) FxToolkit.setupApplication(PopcornTimeApplicationTest.class, "--tv", "--disable-popcorn-keys");
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


}
