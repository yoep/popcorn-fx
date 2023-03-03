package com.github.yoep.popcorn.ui.config;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.updater.UpdateService;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class UpdateConfig {
    @Bean
    public UpdateService updateService(FxLib fxLib,
                                       PopcornFx instance,
                                       PlatformProvider platform) {
        return new UpdateService(fxLib, instance, platform);
    }
}
