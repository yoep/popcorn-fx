package com.github.yoep.video.vlc.conditions;

import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.ApplicationArguments;
import org.springframework.context.annotation.ConditionContext;
import org.springframework.context.annotation.ConfigurationCondition;
import org.springframework.core.type.AnnotatedTypeMetadata;

@Slf4j
public class OnArmCondition implements ConfigurationCondition {
    private static final String ARM_ARCHITECTURE = "arm";

    @Override
    public ConfigurationPhase getConfigurationPhase() {
        return ConfigurationPhase.REGISTER_BEAN;
    }

    @Override
    public boolean matches(ConditionContext context, AnnotatedTypeMetadata metadata) {
        var architecture = System.getProperty("os.arch");
        var beanFactory = context.getBeanFactory();

        if (beanFactory != null) {
            var arguments = beanFactory.getBean(ApplicationArguments.class);

            return arguments.containsOption(Options.FORCE_ARM_PLAYER);
        }

        log.trace("Checking CPU architecture \"{}\" for ARM embedded devices", architecture);
        return architecture.equals(ARM_ARCHITECTURE);
    }
}
