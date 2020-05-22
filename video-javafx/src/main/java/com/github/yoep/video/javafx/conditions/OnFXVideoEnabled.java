package com.github.yoep.video.javafx.conditions;

import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.ApplicationArguments;
import org.springframework.context.annotation.ConditionContext;
import org.springframework.context.annotation.ConfigurationCondition;
import org.springframework.core.type.AnnotatedTypeMetadata;

@Slf4j
public class OnFXVideoEnabled implements ConfigurationCondition {
    private static final String DISABLE_FX_PLAYER = "disable-javafx-video-player";

    @Override
    public ConfigurationPhase getConfigurationPhase() {
        return ConfigurationPhase.REGISTER_BEAN;
    }

    @Override
    public boolean matches(ConditionContext context, AnnotatedTypeMetadata metadata) {
        var beanFactory = context.getBeanFactory();

        if (beanFactory != null) {
            var arguments = beanFactory.getBean(ApplicationArguments.class);

            log.trace("The application started with \"{}\" options", arguments.getOptionNames());
            return !arguments.containsOption(DISABLE_FX_PLAYER);
        }

        log.warn("Unable to process OnFXVideoEnabled condition, bean factory is not present");
        return true;
    }
}
