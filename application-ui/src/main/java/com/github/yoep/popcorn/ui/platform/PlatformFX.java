package com.github.yoep.popcorn.ui.platform;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import javafx.application.ConditionalFeature;
import javafx.application.Platform;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.io.IOException;
import java.util.Objects;

@Slf4j
@RequiredArgsConstructor
public class PlatformFX implements PlatformProvider {
    @Override
    public boolean isTransparentWindowSupported() {
        return Platform.isSupported(ConditionalFeature.TRANSPARENT_WINDOW);
    }

    @Override
    public void runOnRenderer(Runnable runnable) {
        if (Platform.isFxApplicationThread()) {
            runnable.run();
        } else {
            Platform.runLater(runnable);
        }
    }

    @Override
    public boolean launch(String command) {
        Objects.requireNonNull(command, "command cannot be null");
        try {
            Runtime.getRuntime().exec(command);
            return true;
        } catch (IOException e) {
            log.error("Failed to launch process, {}", e.getMessage(), e);
        }

        return false;
    }

    @Override
    public boolean isMac() {
        return com.sun.jna.Platform.isMac();
    }

    @Override
    public void exit(int code) {
        Platform.exit();
        System.exit(code);
    }
}
