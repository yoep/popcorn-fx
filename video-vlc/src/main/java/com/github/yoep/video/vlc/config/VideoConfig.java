package com.github.yoep.video.vlc.config;

import com.github.yoep.popcorn.backend.adapters.video.VideoPlayer;
import com.github.yoep.video.vlc.VideoPlayerVlc;
import com.github.yoep.video.vlc.conditions.ConditionalOnNonArmDevice;
import com.github.yoep.video.vlc.conditions.ConditionalOnVlcInstall;
import com.github.yoep.video.vlc.conditions.ConditionalOnVlcVideoEnabled;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import uk.co.caprica.vlcj.factory.MediaPlayerFactory;
import uk.co.caprica.vlcj.factory.discovery.NativeDiscovery;

@Slf4j
@Configuration
@ConditionalOnNonArmDevice
@ConditionalOnVlcVideoEnabled
public class VideoConfig {
    @Bean
    @Order(Ordered.HIGHEST_PRECEDENCE + 10)
    @ConditionalOnVlcInstall
    public VideoPlayer vlcVideoPlayer(MediaPlayerFactory mediaPlayerFactory) {
        log.info("Using VLC player for video playbacks");
        return new VideoPlayerVlc(mediaPlayerFactory);
    }

    @Bean
    @ConditionalOnVlcInstall
    public MediaPlayerFactory mediaPlayerFactory(NativeDiscovery nativeDiscovery) {
        log.trace("Creating VLC media player factory instance");
        return new MediaPlayerFactory(nativeDiscovery);
    }
}
