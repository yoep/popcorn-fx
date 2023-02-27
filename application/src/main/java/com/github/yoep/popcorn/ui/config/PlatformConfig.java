package com.github.yoep.popcorn.ui.config;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.platform.PlatformFX;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Slf4j
@Configuration
public class PlatformConfig {
    @Bean
    public PlatformProvider platformProvider(FxLib fxLib, PopcornFx instance) {
        return new PlatformFX(fxLib, instance);
    }
}
