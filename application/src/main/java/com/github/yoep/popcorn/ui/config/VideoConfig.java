package com.github.yoep.popcorn.ui.config;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.video.javafx.VideoPlayerFX;
import com.github.yoep.video.javafx.conditions.OnFXVideoEnabled;
import com.github.yoep.video.javafx.conditions.OnMediaSupportedCondition;
import com.github.yoep.video.vlc.VideoPlayerVlc;
import com.github.yoep.video.vlc.VideoPlayerVlcError;
import com.github.yoep.video.youtube.VideoPlayerYoutube;
import com.github.yoep.video.youtube.conditions.OnWebkitSupportedCondition;
import com.github.yoep.video.youtube.conditions.OnYoutubeVideoEnabled;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import uk.co.caprica.vlcj.factory.MediaPlayerFactory;
import uk.co.caprica.vlcj.factory.discovery.NativeDiscovery;

@Slf4j
@Configuration
public class VideoConfig {
    @Bean
    @Order(Ordered.HIGHEST_PRECEDENCE)
    public VideoPlayback youtubeVideoPlayer(FxLib fxLib, PopcornFx instance) {
        if (OnYoutubeVideoEnabled.matches(fxLib, instance) && OnWebkitSupportedCondition.matches()) {
            log.info("Using Youtube player for trailer playbacks");
            return new VideoPlayerYoutube();
        }

        return null;
    }

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

    @Bean
    @Order
    public VideoPlayback javaFxVideoPlayer(FxLib fxLib, PopcornFx instance) {
        if (OnFXVideoEnabled.matches(fxLib, instance) && OnMediaSupportedCondition.matches()) {
            log.info("Using JavaFX player as fallback player");
            return new VideoPlayerFX();
        } else {
            return null;
        }
    }

    private static VideoPlayerVlc createVideoPlayerVlcInstance(NativeDiscovery nativeDiscovery) {
        log.trace("Creating VLC media player factory instance");
        var mediaPlayerFactory = new MediaPlayerFactory(nativeDiscovery);

        log.info("Using VLC player for video playbacks");
        return new VideoPlayerVlc(mediaPlayerFactory);
    }
}
