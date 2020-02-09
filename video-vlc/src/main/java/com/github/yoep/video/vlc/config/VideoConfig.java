package com.github.yoep.video.vlc.config;

import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.vlc.VideoPlayerVlc;
import com.github.yoep.video.vlc.condition.ConditionalOnVlcInstall;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;

@Slf4j
@Configuration
public class VideoConfig {
    @Bean
    @Order(Ordered.HIGHEST_PRECEDENCE + 1)
    @ConditionalOnVlcInstall
    public VideoPlayer vlcVideoPlayer() {
        log.info("Using VLC player for torrent playbacks");
        return new VideoPlayerVlc();
    }
}
