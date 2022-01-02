package com.github.yoep.player.qt.conditions;

import org.springframework.boot.ApplicationArguments;
import org.springframework.context.annotation.ConditionContext;
import org.springframework.context.annotation.ConfigurationCondition;
import org.springframework.core.type.AnnotatedTypeMetadata;

import java.util.Optional;

public class OnQtPlayerEnabledCondition implements ConfigurationCondition {
    static final String DISABLE_OPTION = "disable-qt-player";

    @Override
    public ConfigurationPhase getConfigurationPhase() {
        return ConfigurationPhase.REGISTER_BEAN;
    }

    @Override
    public boolean matches(ConditionContext context, AnnotatedTypeMetadata metadata) {
        var beanFactory = context.getBeanFactory();

        return Optional.ofNullable(beanFactory)
                .map(e -> e.getBean(ApplicationArguments.class))
                .map(e -> !e.containsOption(DISABLE_OPTION))
                .orElse(true);
    }
}
