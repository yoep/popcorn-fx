package com.github.yoep.popcorn.platform;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformInfo;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import javafx.application.ConditionalFeature;
import javafx.application.Platform;
import lombok.extern.slf4j.Slf4j;

import javax.annotation.PreDestroy;
import java.io.IOException;
import java.nio.file.Path;
import java.util.Objects;

@Slf4j
public class PlatformFX implements PlatformProvider {
    private final ApplicationPlatform instance;

    private final PlatformC platform;

    public PlatformFX() {
        instance = ApplicationPlatform.INSTANCE;
        instance.init();
        platform = instance.new_platform_c();
    }

    @Override
    public boolean isTransparentWindowSupported() {
        return Platform.isSupported(ConditionalFeature.TRANSPARENT_WINDOW);
    }

    @Override
    public PlatformInfo platformInfo() {
        try (var info = instance.platform_info()) {
            return info;
        }
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
    public void disableScreensaver() {
        log.debug("Disabling screensaver");
        instance.disable_screensaver(platform);
    }

    @Override
    public boolean launch(Path path) {
        return launch(path.toString());
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
    public void exit() {
        Platform.exit();
    }

    @PreDestroy
    private void onDestroy() {
        instance.enable_screensaver(platform);
    }
}
