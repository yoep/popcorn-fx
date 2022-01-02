package com.github.yoep.player.qt.config;

import com.github.yoep.player.qt.conditions.ConditionalOnQtPlayerEnabled;
import com.github.yoep.player.qt.discovery.QtDiscoveryService;
import com.github.yoep.player.qt.discovery.QtPlayerLibDiscovery;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.DependsOn;

@Slf4j
@Configuration
@ConditionalOnQtPlayerEnabled
public class QtPlayerConfig {
    @Bean
    public QtPlayerLibDiscovery popcornPlayerLibDiscovery() {
        return new QtPlayerLibDiscovery();
    }

    @Bean
    @DependsOn("popcornPlayerLibDiscovery")
    public QtDiscoveryService qtDiscoveryService(PlayerManagerService playerManagerService) {
        return new QtDiscoveryService(playerManagerService);
    }
}
