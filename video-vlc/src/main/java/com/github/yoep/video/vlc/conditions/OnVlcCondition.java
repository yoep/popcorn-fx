package com.github.yoep.video.vlc.conditions;

import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.ConditionContext;
import org.springframework.context.annotation.ConfigurationCondition;
import org.springframework.core.type.AnnotatedTypeMetadata;
import uk.co.caprica.vlcj.factory.discovery.NativeDiscovery;

@Slf4j
class OnVlcCondition implements ConfigurationCondition {
    @Override
    public ConfigurationPhase getConfigurationPhase() {
        return ConfigurationPhase.PARSE_CONFIGURATION;
    }

    @Override
    public boolean matches(ConditionContext context, AnnotatedTypeMetadata metadata) {
        // check if a VLC installation can be found
        var nativeDiscovery = new NativeDiscovery();
        var nativeInstallationDiscovered = nativeDiscovery.discover();

        if (nativeInstallationDiscovered) {
            log.debug("Found native VLC installation at \"{}\"", nativeDiscovery.discoveredPath());
        } else {
            log.warn("VLC installation not found");
        }

        return nativeInstallationDiscovered;
    }
}
