package com.github.yoep.popcorn.backend;

import com.github.yoep.popcorn.backend.config.*;
import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import org.springframework.boot.context.properties.EnableConfigurationProperties;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Import;

@Configuration
@Import({
        MediaConfig.class,
        RestConfig.class,
        SettingsConfig.class,
        SubtitlesConfig.class,
        ThreadConfig.class,
        UtilsConfig.class,
})
@EnableConfigurationProperties({
        PopcornProperties.class
})
public class BackendAutoConfiguration {
}
