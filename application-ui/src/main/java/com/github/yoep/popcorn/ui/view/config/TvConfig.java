package com.github.yoep.popcorn.ui.view.config;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.view.conditions.ConditionalOnTvMode;
import com.github.yoep.popcorn.ui.view.controllers.common.components.TvPosterComponent;
import com.github.yoep.popcorn.ui.view.controllers.tv.components.*;
import com.github.yoep.popcorn.ui.view.services.DetailsComponentService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Scope;

import static org.springframework.beans.factory.config.BeanDefinition.SCOPE_PROTOTYPE;

@Configuration
public class TvConfig {

    @Bean
    @ConditionalOnTvMode
    public TvFilterComponent tvFilterComponent(EventPublisher eventPublisher,
                                               LocaleText localeText,
                                               FxLib fxLib,
                                               PopcornFx instance) {
        return new TvFilterComponent(eventPublisher, localeText, fxLib, instance);
    }

    @Bean
    @ConditionalOnTvMode
    public SystemTimeComponent systemTimeComponent() {
        return new SystemTimeComponent();
    }

    @Bean
    @Scope(SCOPE_PROTOTYPE)
    @ConditionalOnTvMode
    public TvPosterComponent tvPosterComponent(EventPublisher eventPublisher,
                                               ImageService imageService) {
        return new TvPosterComponent(eventPublisher, imageService);
    }

    @Bean
    @ConditionalOnTvMode
    public TvMovieActionsComponent tvMovieActionsComponent(EventPublisher eventPublisher,
                                                           SubtitleService subtitleService,
                                                           LocaleText localeText,
                                                           DetailsComponentService detailsComponentService) {
        return new TvMovieActionsComponent(eventPublisher, subtitleService, localeText, detailsComponentService);
    }

    @Bean
    @ConditionalOnTvMode
    public TvSerieActionsComponent tvSerieActionsComponent(EventPublisher eventPublisher,
                                                           SubtitleService subtitleService) {
        return new TvSerieActionsComponent(eventPublisher, subtitleService);
    }

    @Bean
    @ConditionalOnTvMode
    public TvSidebarSearchComponent tvSidebarSearchComponent(EventPublisher eventPublisher) {
        return new TvSidebarSearchComponent(eventPublisher);
    }

    @Bean
    @ConditionalOnTvMode
    public TvSettingsUiComponent tvSettingsUiComponent(ApplicationConfig applicationConfig,
                                                       LocaleText localeText) {
        return new TvSettingsUiComponent(applicationConfig, localeText);
    }

    @Bean
    @ConditionalOnTvMode
    public TvSettingsSubtitlesComponent tvSettingsSubtitlesComponent(ApplicationConfig applicationConfig) {
        return new TvSettingsSubtitlesComponent(applicationConfig);
    }
}
