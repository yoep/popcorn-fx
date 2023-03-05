package com.github.yoep.popcorn.ui.config;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class EventConfig {
    @Bean
    public EventPublisher eventPublisher() {
        return new EventPublisher();
    }
}
