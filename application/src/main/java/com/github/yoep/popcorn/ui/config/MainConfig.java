package com.github.yoep.popcorn.ui.config;

import com.github.yoep.popcorn.Application;
import com.github.yoep.popcorn.PopcornFx;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class MainConfig {
    @Bean
    public PopcornFx popcornFx() {
        return Application.INSTANCE.new_instance();
    }
}
