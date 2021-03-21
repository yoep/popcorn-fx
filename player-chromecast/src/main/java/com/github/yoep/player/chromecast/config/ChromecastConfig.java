package com.github.yoep.player.chromecast.config;

import com.github.yoep.player.adapter.PlayerService;
import com.github.yoep.player.chromecast.discovery.DiscoveryService;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class ChromecastConfig {
    @Bean
    public DiscoveryService discoveryService(PlayerService playerService) {
        return new DiscoveryService(playerService);
    }
}
