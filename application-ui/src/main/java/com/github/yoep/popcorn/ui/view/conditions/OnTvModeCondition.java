package com.github.yoep.popcorn.ui.view.conditions;

import com.github.yoep.popcorn.backend.lib.FxLibInstance;
import com.github.yoep.popcorn.backend.lib.PopcornFxInstance;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.ConditionContext;
import org.springframework.context.annotation.ConfigurationCondition;
import org.springframework.core.type.AnnotatedTypeMetadata;

@Slf4j
public class OnTvModeCondition implements ConfigurationCondition {
    @Override
    public ConfigurationPhase getConfigurationPhase() {
        return ConfigurationPhase.REGISTER_BEAN;
    }

    @Override
    public boolean matches(ConditionContext conditionContext, AnnotatedTypeMetadata annotatedTypeMetadata) {
        return FxLibInstance.INSTANCE.get().is_tv_mode(PopcornFxInstance.INSTANCE.get()) == 1;
    }
}
