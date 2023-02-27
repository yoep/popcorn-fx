package com.github.yoep.video.youtube.conditions;

import javafx.application.ConditionalFeature;
import javafx.application.Platform;
import lombok.AccessLevel;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;

@Slf4j
@NoArgsConstructor(access = AccessLevel.PRIVATE)
public class OnWebkitSupportedCondition {
    public static boolean matches() {
        boolean supported = Platform.isSupported(ConditionalFeature.WEB);

        if (!supported)
            log.warn("JavaFX web is not supported on this platform, disabling Youtube video player");

        return supported;
    }
}
