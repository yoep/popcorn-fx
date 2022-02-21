package com.github.yoep.popcorn.platform.config;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.platform.PlatformFX;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class PlatformConfig {
    @Bean
    public PlatformProvider platformProviderFX() {
        return new PlatformFX();
    }
}
