package com.github.yoep.popcorn.platform;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformInfo;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import javafx.application.ConditionalFeature;
import javafx.application.Platform;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PreDestroy;
import java.io.IOException;
import java.nio.file.Path;
import java.util.Objects;

@Slf4j
@Service
public class PlatformFX implements PlatformProvider {
    @Override
    public boolean isTransparentWindowSupported() {
        return Platform.isSupported(ConditionalFeature.TRANSPARENT_WINDOW);
    }

    @Override
    public PlatformInfo platformInfo() {
        try (var info = FxLib.INSTANCE.platform_info(PopcornFxInstance.INSTANCE.get())) {
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
        FxLib.INSTANCE.disable_screensaver(PopcornFxInstance.INSTANCE.get());
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
        runOnRenderer(Platform::exit);
    }

    @PreDestroy
    private void onDestroy() {
        FxLib.INSTANCE.enable_screensaver(PopcornFxInstance.INSTANCE.get());
    }
}
