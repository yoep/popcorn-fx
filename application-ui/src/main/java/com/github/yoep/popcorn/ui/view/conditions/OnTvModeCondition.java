package com.github.yoep.popcorn.ui.view.conditions;

import com.github.yoep.popcorn.backend.settings.OptionsService;
import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.ApplicationArguments;
import org.springframework.context.annotation.ConditionContext;
import org.springframework.context.annotation.ConfigurationCondition;
import org.springframework.core.type.AnnotatedTypeMetadata;

@Slf4j
public class OnTvModeCondition implements ConfigurationCondition {
    @Override
    public ConfigurationPhase getConfigurationPhase() {
        return ConfigurationPhase.PARSE_CONFIGURATION;
    }

    @Override
    public boolean matches(ConditionContext conditionContext, AnnotatedTypeMetadata annotatedTypeMetadata) {
        var beanFactory = conditionContext.getBeanFactory();

        if (beanFactory != null) {
            var arguments = beanFactory.getBean(ApplicationArguments.class);

            return arguments.containsOption(OptionsService.TV_MODE_OPTION);
        } else {
            log.warn("Unable to verify TV mode, beanFactory is undefined");
        }

        return false;
    }
}
