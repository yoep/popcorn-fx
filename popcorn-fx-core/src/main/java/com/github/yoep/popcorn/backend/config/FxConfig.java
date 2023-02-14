package com.github.yoep.popcorn.backend.config;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class FxConfig {
    @Bean
    public FxLib fxLib() {
        return FxLib.INSTANCE;
    }

    @Bean
    public PopcornFx fxInstance() {
        return PopcornFxInstance.INSTANCE.get();
    }
}
