package com.github.yoep.video.vlc.conditions;

import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.ApplicationArguments;
import org.springframework.context.annotation.ConditionContext;
import org.springframework.context.annotation.ConfigurationCondition;
import org.springframework.core.type.AnnotatedTypeMetadata;

@Slf4j
public class OnArmVideoEnabled implements ConfigurationCondition {
    static final String DISABLE_OPTION = "disable-arm-video-player";

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
            return !arguments.containsOption(DISABLE_OPTION);
        }

        log.warn("Unable to process OnArmVideoEnabled condition, bean factory is not present");
        return true;
    }
}
