package com.github.yoep.popcorn.backend.config;

import com.github.yoep.popcorn.backend.ApplicationBackend;
import com.github.yoep.popcorn.backend.PopcornFx;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class BackendConfig {
    @Bean
    public PopcornFx popcornFx() {
        return ApplicationBackend.INSTANCE.new_instance();
    }
}
