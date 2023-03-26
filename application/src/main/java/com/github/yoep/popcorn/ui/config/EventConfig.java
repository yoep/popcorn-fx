package com.github.yoep.popcorn.ui.config;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.EventPublisherBridge;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class EventConfig {
    @Bean
    public EventPublisher eventPublisher() {
        return new EventPublisher();
    }

    @Bean
    public EventPublisherBridge eventPublisherBridge(EventPublisher eventPublisher,
                                                     FxLib fxLib,
                                                     PopcornFx instance) {
        return new EventPublisherBridge(eventPublisher, fxLib, instance);
    }
}
