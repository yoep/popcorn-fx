package com.github.yoep.popcorn.backend;

import com.github.yoep.popcorn.backend.config.*;
import com.github.yoep.popcorn.backend.lib.PopcornFxInstance;
import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Import;

import javax.annotation.PreDestroy;

@Configuration
@Import({
        FxConfig.class,
        LoaderConfig.class,
        MediaConfig.class,
        SettingsConfig.class,
        ThreadConfig.class,
        TrackingConfig.class,
        UtilsConfig.class,
})
@ComponentScan("com.github.yoep.popcorn.backend.subtitles")
public class BackendAutoConfiguration {
    @PreDestroy
    public void onDestroy() {
        PopcornFxInstance.INSTANCE.get().dispose();
    }
}
