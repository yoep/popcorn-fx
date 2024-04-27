package com.github.yoep.popcorn;

import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
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
    private FxLib fxLib;
    @Mock
    private PopcornFx popcornFx;
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
    void testCreateLibraryArguments() {
        var args = new String[]{
                "--tv",
                "--kiosk"
        };
        var expectedResult = new String[]{
                "popcorn-fx",
                "--tv",
                "--kiosk"
        };

        var result = PopcornTimeStarter.createLibraryArguments(args);

        var resultArguments = result.args().getStringArray(0);
        assertArrayEquals(expectedResult, resultArguments);
    }

    @Test
    void testOnInit() {
        var ioc = new IoC();
        ioc.registerInstance(fxLib);
        ioc.registerInstance(popcornFx);
        ioc.registerInstance(applicationConfig);
        PopcornTimeStarter.INIT_TORRENT_SERVICES.set(false);

        PopcornTimeStarter.onInit(ioc);
        var result = ioc.getInstance(PopcornPlayer.class);

        assertNotNull(result, "expected the popcorn player to have been initialized");
    }
}