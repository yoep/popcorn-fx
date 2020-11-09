package com.github.yoep.video.vlcnative.config;

import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.vlcnative.PopcornPlayerLibDiscovery;
import com.github.yoep.video.vlcnative.VideoPlayerVlcNative;
import com.github.yoep.video.vlcnative.conditions.ConditionalOnArmDevice;
import com.github.yoep.video.vlcnative.conditions.ConditionalOnArmVideoEnabled;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.DependsOn;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;

@Slf4j
@Configuration
public class VlcNativeConfig {
    @Bean
    @ConditionalOnArmDevice
    @ConditionalOnArmVideoEnabled
    public PopcornPlayerLibDiscovery popcornPlayerLibDiscovery() {
        return new PopcornPlayerLibDiscovery();
    }

    @Bean
    @DependsOn("popcornPlayerLibDiscovery")
    @Order(Ordered.HIGHEST_PRECEDENCE + 8)
    @ConditionalOnArmDevice
    @ConditionalOnArmVideoEnabled
    public VideoPlayer vlcArmVideoPlayer() {
        log.info("Using VLC native player for video playbacks");
        return new VideoPlayerVlcNative();
    }
}
