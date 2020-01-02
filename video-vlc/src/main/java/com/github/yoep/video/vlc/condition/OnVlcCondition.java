package com.github.yoep.video.vlc.condition;

import org.springframework.context.annotation.ConditionContext;
import org.springframework.context.annotation.ConfigurationCondition;
import org.springframework.core.type.AnnotatedTypeMetadata;
import uk.co.caprica.vlcj.factory.discovery.NativeDiscovery;

class OnVlcCondition implements ConfigurationCondition {
    @Override
    public ConfigurationPhase getConfigurationPhase() {
        return ConfigurationPhase.REGISTER_BEAN;
    }

    @Override
    public boolean matches(ConditionContext context, AnnotatedTypeMetadata metadata) {
        // check if a VLC installation can be found
        return new NativeDiscovery().discover();
    }
}
