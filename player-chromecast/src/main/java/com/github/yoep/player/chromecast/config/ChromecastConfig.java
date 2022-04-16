package com.github.yoep.player.chromecast.config;

import com.github.kokorin.jaffree.ffprobe.FFprobe;
import com.github.yoep.player.chromecast.discovery.DiscoveryService;
import com.github.yoep.player.chromecast.discovery.FfmpegDiscovery;
import com.github.yoep.player.chromecast.services.ChromecastService;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.http.client.reactive.ReactorClientHttpConnector;
import org.springframework.web.reactive.function.client.WebClient;
import reactor.netty.http.client.HttpClient;

@Configuration
public class ChromecastConfig {
    @Bean
    public DiscoveryService discoveryService(PlayerManagerService playerService, ChromecastService chromecastService) {
        return new DiscoveryService(playerService, chromecastService);
    }

    @Bean
    public FFprobe fFprobe() {
        return FfmpegDiscovery.discoverProbe();
    }

    @Bean
    public WebClient chromecastWebClient() {
        return WebClient.builder()
                .clientConnector(new ReactorClientHttpConnector(
                        HttpClient.create().followRedirect(true)
                ))
                .build();
    }
}
