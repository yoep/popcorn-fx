package com.github.yoep.video.javafx.config;

import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.javafx.VideoPlayerFX;
import com.github.yoep.video.javafx.condition.ConditionalOnMediaSupported;
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
    @ConditionalOnMediaSupported
    public VideoPlayer javaFxVideoPlayer() {
        log.info("Using JavaFX player as fallback player");
        return new VideoPlayerFX();
    }
}
