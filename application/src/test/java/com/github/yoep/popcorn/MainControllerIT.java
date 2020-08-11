package com.github.yoep.popcorn;

import org.junit.jupiter.api.Test;
import org.testfx.api.FxAssert;
import org.testfx.matcher.base.NodeMatchers;

public class MainControllerIT extends TestFxBase {
    static final String CONTENT_PANE_ID = "#contentPane";

    @Test
    public void testStartup_whenNoNonOptionArgumentAreGiven_shouldShowContentAsStartScreen() {
        FxAssert.verifyThat(CONTENT_PANE_ID, NodeMatchers.isNotNull());
    }
}
