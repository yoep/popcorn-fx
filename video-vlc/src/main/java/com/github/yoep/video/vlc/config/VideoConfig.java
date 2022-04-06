package com.github.yoep.video.vlc.config;

import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.video.vlc.VideoPlayerVlc;
import com.github.yoep.video.vlc.VideoPlayerVlcError;
import com.github.yoep.vlc.conditions.ConditionalOnVlcVideoEnabled;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import uk.co.caprica.vlcj.factory.MediaPlayerFactory;
import uk.co.caprica.vlcj.factory.discovery.NativeDiscovery;

@Slf4j
@Configuration
@ConditionalOnVlcVideoEnabled
public class VideoConfig {
    @Bean
    @Order(Ordered.HIGHEST_PRECEDENCE + 10)
    public VideoPlayback vlcVideoPlayer(NativeDiscovery nativeDiscovery) {
        if (nativeDiscovery.discover()) {
            log.debug("Discovered VLC library at {}", nativeDiscovery.discoveredPath());
            return createVideoPlayerVlcInstance(nativeDiscovery);
        } else {
            log.warn("Failed to discover VLC library");
            return new VideoPlayerVlcError();
        }
    }

    private static VideoPlayerVlc createVideoPlayerVlcInstance(NativeDiscovery nativeDiscovery) {
        log.trace("Creating VLC media player factory instance");
        var mediaPlayerFactory = new MediaPlayerFactory(nativeDiscovery);

        log.info("Using VLC player for video playbacks");
        return new VideoPlayerVlc(mediaPlayerFactory);
    }
}
