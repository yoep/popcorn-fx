package com.github.yoep.popcorn.backend;

import com.github.yoep.popcorn.backend.config.*;
import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.lib.PopcornFxInstance;
import org.springframework.boot.context.properties.EnableConfigurationProperties;
import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Import;

import javax.annotation.PreDestroy;

@Configuration
@Import({
        FxConfig.class,
        MediaConfig.class,
        RestConfig.class,
        SettingsConfig.class,
        ThreadConfig.class,
        TorrentStreamConfig.class,
        UtilsConfig.class,
})
@EnableConfigurationProperties({
        PopcornProperties.class
})
@ComponentScan("com.github.yoep.popcorn.backend.subtitles")
public class BackendAutoConfiguration {
    @PreDestroy
    public void onDestroy() {
        PopcornFxInstance.INSTANCE.get().dispose();
    }
}
