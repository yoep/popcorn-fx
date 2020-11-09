package com.github.yoep.video.vlc.config;

import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.vlc.VideoPlayerVlc;
import com.github.yoep.video.vlc.conditions.ConditionalOnVlcInstall;
import com.github.yoep.video.vlc.conditions.ConditionalOnVlcVideoEnabled;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import uk.co.caprica.vlcj.factory.discovery.NativeDiscovery;

@Slf4j
@Configuration
public class VideoConfig {
    @Bean
    @Order(Ordered.HIGHEST_PRECEDENCE + 10)
    @ConditionalOnVlcInstall
    @ConditionalOnVlcVideoEnabled
    public VideoPlayer vlcVideoPlayer(NativeDiscovery nativeDiscovery) {
        log.info("Using VLC player for video playbacks");
        return new VideoPlayerVlc(nativeDiscovery);
    }
}
