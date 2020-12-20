package com.github.yoep.video.vlc.conditions;

import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.ConditionContext;
import org.springframework.context.annotation.ConfigurationCondition;
import org.springframework.core.type.AnnotatedTypeMetadata;

@Slf4j
public class OnNonArmCondition implements ConfigurationCondition {
    private static final String ARM_ARCHITECTURE = "arm";

    @Override
    public ConfigurationPhase getConfigurationPhase() {
        return ConfigurationPhase.REGISTER_BEAN;
    }

    @Override
    public boolean matches(ConditionContext context, AnnotatedTypeMetadata metadata) {
        var architecture = System.getProperty("os.arch");

        log.trace("Checking CPU architecture \"{}\" for non ARM embedded devices", architecture);
        return !architecture.equals(ARM_ARCHITECTURE);
    }
}
