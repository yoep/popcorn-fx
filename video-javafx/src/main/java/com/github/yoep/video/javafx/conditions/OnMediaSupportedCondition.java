package com.github.yoep.video.javafx.conditions;

import javafx.application.ConditionalFeature;
import javafx.application.Platform;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;

@Slf4j
@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class OnMediaSupportedCondition {
    public static boolean matches() {
        boolean supported = Platform.isSupported(ConditionalFeature.WEB);

        if (!supported)
            log.warn("JavaFX media is not supported on this platform, disabling JavaFX player as fallback option");

        return supported;
    }
}
