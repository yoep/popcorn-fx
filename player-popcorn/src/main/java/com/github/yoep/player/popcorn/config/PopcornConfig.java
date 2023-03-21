package com.github.yoep.player.popcorn.config;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.player.popcorn.controllers.components.DesktopHeaderActionsComponent;
import com.github.yoep.player.popcorn.controllers.components.DesktopPlayerControlsComponent;
import com.github.yoep.player.popcorn.controllers.components.TvPlayerControlsComponent;
import com.github.yoep.player.popcorn.services.PlayerControlsService;
import com.github.yoep.player.popcorn.services.PlayerHeaderService;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.ui.view.conditions.ConditionalOnDesktopMode;
import com.github.yoep.popcorn.ui.view.conditions.ConditionalOnTvMode;
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
                                                               PlayerControlsService playerControlsService) {
        return new TvPlayerControlsComponent(eventPublisher, playerControlsService);
    }
}
