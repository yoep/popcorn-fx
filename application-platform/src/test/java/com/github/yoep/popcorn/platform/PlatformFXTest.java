package com.github.yoep.popcorn.platform;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.platform.PlatformInfo;
import javafx.application.ConditionalFeature;
import javafx.application.Platform;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class PlatformFXTest {
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx instance;
    @InjectMocks
    private PlatformFX platform;

    @Test
    void testIsTransparentWindowSupported() {
        var future = new CompletableFuture<Boolean>();

        Platform.runLater(() -> {
            var expectedResult = Platform.isSupported(ConditionalFeature.TRANSPARENT_WINDOW);
            var result = platform.isTransparentWindowSupported();

            future.complete(expectedResult == result);
        });

        assertTrue(future.join());
    }

    @Test
    void testPlatformInfo() {
        var info = mock(PlatformInfo.class);
        when(fxLib.platform_info(instance)).thenReturn(info);

        var result = platform.platformInfo();

        assertEquals(info, result);
    }

    @Test
    void testRunOnRenderer() throws ExecutionException, InterruptedException, TimeoutException {
        var future = new CompletableFuture<>();

        platform.runOnRenderer(() -> future.complete(null));

        future.get(1, TimeUnit.SECONDS);
    }
}