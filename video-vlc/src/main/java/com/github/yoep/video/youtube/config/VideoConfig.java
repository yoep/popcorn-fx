package com.github.yoep.video.youtube.config;

import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.youtube.VideoPlayerVlc;
import com.github.yoep.video.youtube.condition.ConditionalOnVlcInstall;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;

@Slf4j
@Configuration
public class VideoConfig {
    @Bean
    @Order(Ordered.HIGHEST_PRECEDENCE)
    @ConditionalOnVlcInstall
    public VideoPlayer videoPlayer() {
        log.info("Using VLC player as video player");
        return new VideoPlayerVlc();
    }
}
