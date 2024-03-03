package com.github.yoep.popcorn.backend.config;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.controls.PlaybackControlsService;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.player.PlayerEventService;
import com.github.yoep.popcorn.backend.player.PlayerManagerServiceImpl;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.Configuration;

@Configuration
@ComponentScan({
        "com.github.yoep.popcorn.backend.media.favorites",
        "com.github.yoep.popcorn.backend.media.providers",
        "com.github.yoep.popcorn.backend.media.resume",
        "com.github.yoep.popcorn.backend.media.watched",
})
public class MediaConfig {
    @Bean
    public PlaybackControlsService playbackControlsService(FxLib fxLib,
                                                           PopcornFx instance,
                                                           PlayerManagerService playerManagerService,
                                                           PlayerEventService playerEventService) {
        return new PlaybackControlsService(fxLib, instance, playerManagerService, playerEventService);
    }

    @Bean
    public PlayerEventService playerManagerService(PlayerManagerService playerService,
                                                   EventPublisher eventPublisher) {
        return new PlayerEventService(playerService, eventPublisher);
    }

    @Bean
    public PlaylistManager playlistManager(FxLib fxLib, PopcornFx instance, ApplicationConfig applicationConfig) {
        return new PlaylistManager(fxLib, instance, applicationConfig);
    }

    @Bean
    public PlayerManagerService playerService(FxLib fxLib, PopcornFx instance, EventPublisher eventPublisher) {
        return new PlayerManagerServiceImpl(fxLib, instance, eventPublisher);
    }
}
