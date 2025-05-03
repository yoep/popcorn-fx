package com.github.yoep.popcorn.ui.platform;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import javafx.application.ConditionalFeature;
import javafx.application.Platform;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

@Slf4j
@RequiredArgsConstructor
public class PlatformFX implements PlatformProvider {
    @Override
    public boolean isTransparentWindowSupported() {
        return Platform.isSupported(ConditionalFeature.TRANSPARENT_WINDOW);
    }

    @Override
    public boolean isMac() {
        var osName = System.getProperty("os.name").toLowerCase();
        return osName.contains("mac");
    }

    @Override
    public void exit(int code) {
        Platform.exit();
        System.exit(code);
    }
}
