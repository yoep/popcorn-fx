package com.github.yoep.player.chromecast.config;

import com.github.yoep.player.chromecast.services.TranscodeService;
import com.github.yoep.player.chromecast.transcode.NoOpTranscodeService;
import com.github.yoep.player.chromecast.transcode.VlcTranscodeService;
import com.github.yoep.vlc.conditions.ConditionalOnVlcVideoEnabled;
import org.springframework.boot.autoconfigure.condition.ConditionalOnMissingBean;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import uk.co.caprica.vlcj.factory.MediaPlayerFactory;
import uk.co.caprica.vlcj.factory.discovery.NativeDiscovery;

@Configuration
public class TranscodeConfig {
    @Bean
    @Order(Ordered.LOWEST_PRECEDENCE - 20)
    @ConditionalOnVlcVideoEnabled
    public TranscodeService vlcTranscodeService(NativeDiscovery discovery) {
        return new VlcTranscodeService(new MediaPlayerFactory(discovery));
    }

    @Bean
    @Order
    @ConditionalOnMissingBean(TranscodeService.class)
    public TranscodeService noOpTranscodeService() {
        return new NoOpTranscodeService();
    }
}
