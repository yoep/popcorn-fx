package com.github.yoep.popcorn.ui.config;

import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.video.youtube.VideoPlayerYoutube;
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
    public VideoPlayback youtubeVideoPlayer() {
        log.info("Using Youtube player for trailer playbacks");
        return new VideoPlayerYoutube();
    }
}
