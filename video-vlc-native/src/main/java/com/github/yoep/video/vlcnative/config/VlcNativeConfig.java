package com.github.yoep.video.vlcnative.config;

import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.vlcnative.VideoPlayerVlcNative;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;

@Slf4j
@Configuration
public class VlcNativeConfig {
    @Bean
    @Order(Ordered.HIGHEST_PRECEDENCE + 8)
    public VideoPlayer vlcArmVideoPlayer() {
        log.info("Using VLC native player for video playbacks");
        return new VideoPlayerVlcNative();
    }
}
