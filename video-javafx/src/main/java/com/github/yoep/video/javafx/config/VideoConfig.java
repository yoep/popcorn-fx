package com.github.yoep.video.javafx.config;

import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.javafx.VideoPlayerFX;
import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.autoconfigure.condition.ConditionalOnMissingBean;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.annotation.Order;

@Slf4j
@Configuration
public class VideoConfig {
    @Bean
    @Order
    @ConditionalOnMissingBean
    public VideoPlayer video() {
        log.warn("Using JavaFX video player as fallback");
        return new VideoPlayerFX();
    }
}
