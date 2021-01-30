package com.github.yoep.popcorn.ui.keys.conditions;

import com.github.yoep.popcorn.ui.keys.PopcornKeysLibDiscovery;
import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.BeanFactory;
import org.springframework.boot.ApplicationArguments;
import org.springframework.context.annotation.ConditionContext;
import org.springframework.context.annotation.ConfigurationCondition;
import org.springframework.core.type.AnnotatedTypeMetadata;

@Slf4j
public class OnPopcornKeys implements ConfigurationCondition {
    private static final String DISABLE_POPCORN_KEYS = "disable-popcorn-keys";

    @Override
    public ConfigurationPhase getConfigurationPhase() {
        return ConfigurationPhase.REGISTER_BEAN;
    }

    @Override
    public boolean matches(ConditionContext context, AnnotatedTypeMetadata metadata) {
        var beanFactory = context.getBeanFactory();

        if (beanFactory != null) {
            // check if the popcorn keys is disabled through argument options
            if (isDisabled(beanFactory)) {
                log.trace("Popcorn keys has been disabled through the argument options, skipping library check");
                return false;
            }

            return isLibraryFound(beanFactory);
        } else {
            log.warn("Unable to verify Popcorn Keys library condition, bean factory is missing");
        }

        return false;
    }

    private boolean isDisabled(BeanFactory beanFactory) {
        var arguments = beanFactory.getBean(ApplicationArguments.class);

        return arguments.containsOption(DISABLE_POPCORN_KEYS);
    }

    private boolean isLibraryFound(BeanFactory beanFactory) {
        var discovery = beanFactory.getBean(PopcornKeysLibDiscovery.class);
        var libraryFound = discovery.libraryFound();

        if (!libraryFound) {
            log.warn("Popcorn Keys library is missing, global media keys will not be enabled.\n" +
                    "To enabled keyboard media keys, make sure the \"{}\" library is present", PopcornKeysLibDiscovery.LIBRARY_NAME);
        }

        return libraryFound;
    }
}
