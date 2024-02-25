package com.github.yoep.popcorn.backend.config;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.tracking.AuthorizationOpenCallback;
import com.github.yoep.popcorn.backend.tracking.TrackingService;
import com.github.yoep.popcorn.backend.tracking.TraktTrackingService;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

@Configuration
public class TrackingConfig {
    @Bean
    public TrackingService trackingService(FxLib fxLib,
                                           PopcornFx instance,
                                           AuthorizationOpenCallback callback) {
        return new TraktTrackingService(fxLib, instance, callback);
    }
}
