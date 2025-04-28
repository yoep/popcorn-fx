package com.github.yoep.popcorn;

import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.IoC;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class PopcornTimeStarterTest {
    @Mock
    private FxChannel fxChannel;
    @Mock
    private ApplicationConfig applicationConfig;

    @Test
    void testCreateApplicationArguments() {
        var urlArguments = "http://localhost/MyVideo.mp4";
        var args = new String[]{
                "--tv",
                urlArguments
        };
        var expectedResult = new String[]{
                urlArguments
        };

        var result = PopcornTimeStarter.createApplicationArguments(args);

        assertArrayEquals(expectedResult, result.args());
    }

    @Test
    void testOnInit() {
        var ioc = new IoC();
        ioc.registerInstance(fxChannel);
        ioc.registerInstance(applicationConfig);

        PopcornTimeStarter.onInit(ioc);
        var result = ioc.getInstance(PopcornPlayer.class);

        assertNotNull(result, "expected the popcorn player to have been initialized");
    }
}