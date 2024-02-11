package com.github.yoep.player.popcorn.config;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.player.popcorn.controllers.components.*;
import com.github.yoep.player.popcorn.services.PlayerControlsService;
import com.github.yoep.player.popcorn.services.PlayerHeaderService;
import com.github.yoep.player.popcorn.services.PlayerSubtitleService;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.ui.view.conditions.ConditionalOnDesktopMode;
import com.github.yoep.popcorn.ui.view.conditions.ConditionalOnTvMode;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.ComponentScan;
import org.springframework.context.annotation.Configuration;

@Configuration
@ComponentScan({
        "com.github.yoep.player.popcorn.controllers",
        "com.github.yoep.player.popcorn.services",
        "com.github.yoep.player.popcorn.player",
        "com.github.yoep.player.popcorn.subtitles"
})
public class PopcornConfig {
    @Bean
    @ConditionalOnDesktopMode
    public DesktopHeaderActionsComponent desktopHeaderActionsComponent(PlayerHeaderService headerService,
                                                                       EventPublisher eventPublisher,
                                                                       LocaleText localeText) {
        return new DesktopHeaderActionsComponent(headerService, eventPublisher, localeText);
    }

    @Bean
    @ConditionalOnDesktopMode
    public DesktopPlayerControlsComponent desktopPlayerControlsComponent(PlayerControlsService playerControlsService,
                                                                         EventPublisher eventPublisher) {
        return new DesktopPlayerControlsComponent(playerControlsService, eventPublisher);
    }

    @Bean
    @ConditionalOnTvMode
    public TvPlayerControlsComponent tvPlayerControlsComponent(EventPublisher eventPublisher,
                                                               PlayerControlsService playerControlsService,
                                                               PlayerSubtitleService subtitleService,
                                                               LocaleText localeText) {
        return new TvPlayerControlsComponent(eventPublisher, playerControlsService, subtitleService, localeText);
    }

    @Bean
    public PlayerHeaderComponent playerHeaderComponent(EventPublisher eventPublisher,
                                                       ViewLoader viewLoader) {
        return new PlayerHeaderComponent(eventPublisher, viewLoader);
    }

    @Bean
    public PlayerSubtitleComponent playerSubtitleComponent(PlayerSubtitleService subtitleService,
                                                           LocaleText localeText) {
        return new PlayerSubtitleComponent(subtitleService, localeText);
    }

    @Bean
    public PlayerPlaylistComponent playerPlaylistComponent(PlaylistManager playlistManager,
                                                           ViewLoader viewLoader,
                                                           ImageService imageService) {
        return new PlayerPlaylistComponent(playlistManager, viewLoader, imageService);
    }
}
