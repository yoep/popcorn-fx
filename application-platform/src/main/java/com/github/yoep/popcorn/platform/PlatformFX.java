package com.github.yoep.popcorn.platform;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformInfo;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformType;
import com.github.yoep.popcorn.platform.jna.ApplicationPlatform;
import com.github.yoep.popcorn.platform.jna.linux.LinuxUtils;
import com.github.yoep.popcorn.platform.jna.macos.MacOsUtils;
import com.github.yoep.popcorn.platform.jna.win32.Win32Utils;
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

    public PlatformFX() {
        instance = ApplicationPlatform.INSTANCE;
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
        switch (platformInfo().getType()) {
            case WINDOWS -> Win32Utils.disableScreensaver();
            case MAC -> MacOsUtils.disableScreensaver();
            default -> LinuxUtils.disableScreensaver();
        }
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
        if (platformInfo().getType() == PlatformType.WINDOWS) {
            Win32Utils.allowScreensaver();
        }
    }
}
