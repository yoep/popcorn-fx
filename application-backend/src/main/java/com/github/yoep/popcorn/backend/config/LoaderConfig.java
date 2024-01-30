package com.github.yoep.popcorn.backend.config;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.loader.LoaderService;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class LoaderConfig {
    @Bean
    public LoaderService mediaLoaderService(FxLib fxLib, PopcornFx instance) {
        return new LoaderService(fxLib, instance);
    }
}
