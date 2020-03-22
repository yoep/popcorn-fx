package com.github.yoep.popcorn;

import org.junit.Ignore;
import org.junit.Test;
import org.testfx.api.FxAssert;
import org.testfx.matcher.base.NodeMatchers;

@Ignore
public class MainControllerTest extends TestFxBase {
    static final String CONTENT_PANE_ID = "#contentPane";
    static final String PLAYER_PANE_ID = "#playerPane";

    @Test
    public void testStartup_whenNoNonOptionArgumentAreGiven_shouldShowContentAsStartScreen() {
        FxAssert.verifyThat(CONTENT_PANE_ID, NodeMatchers.isNotNull());
    }

    @Test
    public void testStartup_whenNonOptionArgumentIsGiven_shouldShowPlayerAsStartScreen() {
        FxAssert.verifyThat(PLAYER_PANE_ID, NodeMatchers.isNotNull());
    }
}
