package com.github.yoep.popcorn.ui.environment;

import com.github.yoep.popcorn.backend.platform.PlatformProvider;
import javafx.application.ConditionalFeature;
import javafx.application.Platform;
import org.springframework.stereotype.Component;

@Component
public class PlatformJavaFX implements PlatformProvider {
    @Override
    public boolean isTransparentWindowSupported() {
        return Platform.isSupported(ConditionalFeature.TRANSPARENT_WINDOW);
    }

    @Override
    public void runOnRenderer(Runnable runnable) {
        Platform.runLater(runnable);
    }
}