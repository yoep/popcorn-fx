package com.github.yoep.video.youtube.condition;

import javafx.application.ConditionalFeature;
import javafx.application.Platform;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.ConditionContext;
import org.springframework.context.annotation.ConfigurationCondition;
import org.springframework.core.type.AnnotatedTypeMetadata;

@Slf4j
public class OnWebkitSupportedCondition implements ConfigurationCondition {

    @Override
    public ConfigurationPhase getConfigurationPhase() {
        return ConfigurationPhase.REGISTER_BEAN;
    }

    @Override
    public boolean matches(ConditionContext context, AnnotatedTypeMetadata metadata) {
        boolean supported = Platform.isSupported(ConditionalFeature.WEB);

        if (!supported)
            log.warn("JavaFX web is not supported on this platform, disabling Youtube video player");

        return supported;
    }
}
