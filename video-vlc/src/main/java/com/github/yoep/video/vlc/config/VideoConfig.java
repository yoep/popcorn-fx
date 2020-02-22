package com.github.yoep.video.vlc.config;

import com.github.yoep.video.adapter.VideoPlayer;
import com.github.yoep.video.vlc.VideoPlayerVlc;
import com.github.yoep.video.vlc.VideoPlayerVlcArm;
import com.github.yoep.video.vlc.conditions.ConditionalOnArmDevice;
import com.github.yoep.video.vlc.conditions.ConditionalOnArmVideoEnabled;
import com.github.yoep.video.vlc.conditions.ConditionalOnVlcInstall;
import lombok.extern.slf4j.Slf4j;
import org.springframework.boot.autoconfigure.condition.ConditionalOnMissingBean;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;

@Slf4j
@Configuration
@ConditionalOnVlcInstall
public class VideoConfig {
    @Bean
    @Order(Ordered.HIGHEST_PRECEDENCE + 9)
//    @ConditionalOnArmDevice
    @ConditionalOnArmVideoEnabled
    public VideoPlayer vlcArmVideoPlayer() {
        log.info("Using VLC ARM player for video playbacks");
        return new VideoPlayerVlcArm();
    }

    @Bean
    @Order(Ordered.HIGHEST_PRECEDENCE + 10)
    @ConditionalOnMissingBean(name = {"vlcArmVideoPlayer"})
    public VideoPlayer vlcVideoPlayer() {
        log.info("Using VLC player for video playbacks");
        return new VideoPlayerVlc();
    }
}
