package com.github.yoep.popcorn.ui.view.config;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.spring.boot.javafx.view.ViewManager;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.view.PopcornViewLoader;
import com.github.yoep.popcorn.ui.view.conditions.ConditionalOnDesktopMode;
import com.github.yoep.popcorn.ui.view.controllers.ContentSectionController;
import com.github.yoep.popcorn.ui.view.controllers.MainController;
import com.github.yoep.popcorn.ui.view.controllers.common.components.DesktopPosterComponent;
import com.github.yoep.popcorn.ui.view.controllers.common.sections.DetailsSectionController;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.*;
import com.github.yoep.popcorn.ui.view.services.*;
import org.springframework.boot.ApplicationArguments;
import org.springframework.context.ApplicationContext;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.annotation.Scope;
import org.springframework.core.task.TaskExecutor;
import org.springframework.scheduling.annotation.EnableScheduling;

import static org.springframework.beans.factory.config.BeanDefinition.SCOPE_PROTOTYPE;

@Configuration
@EnableScheduling
public class ViewConfig {
    @Bean
    public MainController mainController(EventPublisher eventPublisher,
                                         ViewLoader viewLoader,
                                         ApplicationArguments arguments,
                                         UrlService urlService,
                                         ApplicationConfig settingsService,
                                         PlatformProvider platformProvider) {
        return new MainController(eventPublisher, viewLoader, arguments, urlService, settingsService, platformProvider);
    }

    @Bean
    public ContentSectionController contentSectionController(ViewLoader viewLoader,
                                                             LocaleText localeText,
                                                             EventPublisher eventPublisher,
                                                             MaximizeService maximizeService,
                                                             ApplicationConfig applicationConfig) {
        return new ContentSectionController(viewLoader, localeText, eventPublisher, maximizeService, applicationConfig);
    }

    @Bean
    public DetailsSectionController detailsSectionController(EventPublisher eventPublisher,
                                                             ViewLoader viewLoader,
                                                             TaskExecutor taskExecutor) {
        return new DetailsSectionController(eventPublisher, viewLoader, taskExecutor);
    }

    @Bean
    public ViewLoader viewLoader(ApplicationContext applicationContext,
                                 ViewManager viewManager,
                                 LocaleText localeText,
                                 ApplicationConfig applicationConfig) {
        return new PopcornViewLoader(applicationContext, viewManager, localeText, applicationConfig);
    }

    @Bean
    @ConditionalOnDesktopMode
    public DesktopFilterComponent desktopFilterComponent(LocaleText localeText,
                                                         EventPublisher eventPublisher,
                                                         FxLib fxLib,
                                                         PopcornFx instance) {
        return new DesktopFilterComponent(localeText, eventPublisher, fxLib, instance);
    }

    @Bean
    @ConditionalOnDesktopMode
    public WindowComponent windowComponent(MaximizeService maximizeService,
                                           PlatformProvider platformProvider) {
        return new WindowComponent(maximizeService, platformProvider);
    }


    @Bean
    @Scope(SCOPE_PROTOTYPE)
    @ConditionalOnDesktopMode
    public DesktopPosterComponent desktopPosterComponent(EventPublisher eventPublisher,
                                                         ImageService imageService,
                                                         FavoriteService favoriteService,
                                                         WatchedService watchedService,
                                                         LocaleText localeText) {
        return new DesktopPosterComponent(eventPublisher, imageService, favoriteService, watchedService, localeText);
    }


    @Bean
    @ConditionalOnDesktopMode
    public DesktopMovieActionsComponent desktopMovieActionsComponent(PlayerManagerService playerService,
                                                                     PlaylistManager playlistManager,
                                                                     EventPublisher eventPublisher,
                                                                     LocaleText localeText,
                                                                     SubtitleService subtitleService,
                                                                     DetailsComponentService detailsComponentService,
                                                                     DesktopMovieQualityComponent desktopMovieQualityComponent) {
        return new DesktopMovieActionsComponent(playerService, playlistManager, eventPublisher, localeText, subtitleService, detailsComponentService,
                desktopMovieQualityComponent);
    }

    @Bean
    @ConditionalOnDesktopMode
    public DesktopMovieQualityComponent desktopMovieQualityComponent(EventPublisher eventPublisher,
                                                                     VideoQualityService videoQualityService) {
        return new DesktopMovieQualityComponent(eventPublisher, videoQualityService);
    }

    @Bean
    @ConditionalOnDesktopMode
    public DesktopSidebarSearchComponent desktopSidebarSearchComponent(EventPublisher eventPublisher) {
        return new DesktopSidebarSearchComponent(eventPublisher);
    }

    @Bean
    @ConditionalOnDesktopMode
    public DesktopSerieActionsComponent desktopSerieActionsComponent(EventPublisher eventPublisher,
                                                                     PlayerManagerService playerManagerService,
                                                                     SubtitleService subtitleService,
                                                                     DesktopSerieQualityComponent desktopSerieQualityComponent,
                                                                     PlaylistManager playlistManager) {
        return new DesktopSerieActionsComponent(eventPublisher, playerManagerService, subtitleService, desktopSerieQualityComponent, playlistManager);
    }

    @Bean
    @ConditionalOnDesktopMode
    public DesktopSerieQualityComponent desktopSerieQualityComponent(EventPublisher eventPublisher,
                                                                     VideoQualityService videoQualityService) {
        return new DesktopSerieQualityComponent(eventPublisher, videoQualityService);
    }
}
