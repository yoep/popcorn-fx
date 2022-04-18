package com.github.yoep.player.vlc.config;

import com.github.yoep.player.vlc.VlcPlayerConstants;
import com.github.yoep.player.vlc.discovery.VlcDiscovery;
import com.github.yoep.player.vlc.services.VlcPlayerService;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.web.reactive.function.client.WebClient;

@Configuration
public class VlcDiscoveryConfig {
    @Bean
    public VlcDiscovery vlcDiscovery(PlayerManagerService playerManagerService, VlcPlayerService vlcPlayerService) {
        return new VlcDiscovery(playerManagerService, vlcPlayerService);
    }

    @Bean
    public WebClient vlcWebClient() {
        return WebClient.builder()
                .defaultHeaders(header -> header.setBasicAuth("", VlcPlayerConstants.PASSWORD))
                .build();
    }
}
