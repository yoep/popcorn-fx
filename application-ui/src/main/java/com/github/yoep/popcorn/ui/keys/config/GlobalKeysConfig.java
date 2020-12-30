package com.github.yoep.popcorn.ui.keys.config;

import com.github.yoep.popcorn.ui.keys.*;
import com.github.yoep.popcorn.ui.keys.conditions.ConditionalOnPopcornKeys;
import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.autoconfigure.condition.ConditionalOnBean;
import org.springframework.boot.autoconfigure.condition.ConditionalOnMissingBean;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Slf4j
@Configuration
public class GlobalKeysConfig {
    @Bean
    @ConditionalOnPopcornKeys
    public PopcornKeys popcornKeys() {
        var level = getLogLevel();
        var args = new String[]{"PopcornKeys", "-l", level};

        return new PopcornKeysImpl(args);
    }

    @Bean
    @ConditionalOnBean(PopcornKeys.class)
    public GlobalKeysService popcornGlobalKeysService(PopcornKeys popcornKeys) {
        return new PopcornGlobalKeysService(popcornKeys);
    }

    @Bean
    @ConditionalOnMissingBean(PopcornKeys.class)
    public GlobalKeysService dummyGlobalKeysService() {
        // create a dummy if the popcorn keys library couldn't be loaded
        // this should prevent the application from not starting
        return new DummyGlobalKeysService();
    }

    private String getLogLevel() {
        if (log.isTraceEnabled()) {
            return "trace";
        } else if (log.isDebugEnabled()) {
            return "debug";
        }

        return "info";
    }
}
