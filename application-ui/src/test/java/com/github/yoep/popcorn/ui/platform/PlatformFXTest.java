package com.github.yoep.popcorn.ui.platform;

import javafx.application.ConditionalFeature;
import javafx.application.Platform;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static org.junit.jupiter.api.Assertions.assertTrue;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class PlatformFXTest {
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
    void testRunOnRenderer() throws ExecutionException, InterruptedException, TimeoutException {
        var future = new CompletableFuture<>();

        platform.runOnRenderer(() -> future.complete(null));

        future.get(1, TimeUnit.SECONDS);
    }
}