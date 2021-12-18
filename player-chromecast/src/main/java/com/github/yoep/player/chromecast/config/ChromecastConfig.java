package com.github.yoep.player.chromecast.config;

import com.github.yoep.player.chromecast.discovery.DiscoveryService;
import com.github.yoep.player.chromecast.services.MetaDataService;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.web.reactive.function.client.WebClient;

@Configuration
public class ChromecastConfig {
    @Bean
    public DiscoveryService discoveryService(PlayerManagerService playerService, MetaDataService contentTypeService) {
        return new DiscoveryService(playerService, contentTypeService);
    }

    @Bean
    public WebClient chromecastWebClient() {
        return WebClient.create();
    }
}
