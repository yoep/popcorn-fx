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
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.OptionsService;
import com.github.yoep.popcorn.ui.view.PopcornViewLoader;
import com.github.yoep.popcorn.ui.view.conditions.ConditionalOnDesktopMode;
import com.github.yoep.popcorn.ui.view.conditions.ConditionalOnTvMode;
import com.github.yoep.popcorn.ui.view.controllers.ContentSectionController;
import com.github.yoep.popcorn.ui.view.controllers.MainController;
import com.github.yoep.popcorn.ui.view.controllers.common.sections.DetailsSectionController;
import com.github.yoep.popcorn.ui.view.controllers.components.DesktopPosterComponent;
import com.github.yoep.popcorn.ui.view.controllers.components.TvPosterComponent;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.DesktopFilterComponent;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.DesktopMovieActionsComponent;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.DesktopSidebarSearchComponent;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.WindowComponent;
import com.github.yoep.popcorn.ui.view.controllers.tv.components.SystemTimeComponent;
import com.github.yoep.popcorn.ui.view.controllers.tv.components.TvFilterComponent;
import com.github.yoep.popcorn.ui.view.controllers.tv.components.TvMovieActionsComponent;
import com.github.yoep.popcorn.ui.view.controllers.tv.components.TvSidebarSearchComponent;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.popcorn.ui.view.services.MaximizeService;
import com.github.yoep.popcorn.ui.view.services.UrlService;
import org.springframework.boot.ApplicationArguments;
import org.springframework.context.ApplicationContext;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.task.TaskExecutor;
import org.springframework.scheduling.annotation.EnableScheduling;

@Configuration
@EnableScheduling
public class ViewConfig {
    @Bean
    public MainController mainController(EventPublisher eventPublisher,
                                         ViewLoader viewLoader,
                                         ApplicationArguments arguments,
                                         UrlService urlService,
                                         ApplicationConfig settingsService,
                                         OptionsService optionsService,
                                         TaskExecutor taskExecutor) {
        return new MainController(eventPublisher, viewLoader, arguments, urlService, settingsService, optionsService, taskExecutor);
    }

    @Bean
    public ContentSectionController contentSectionController(ViewLoader viewLoader,
                                                             LocaleText localeText,
                                                             EventPublisher eventPublisher,
                                                             MaximizeService maximizeService,
                                                             OptionsService optionsService) {
        return new ContentSectionController(viewLoader, localeText, eventPublisher, maximizeService, optionsService);
    }

    @Bean
    public DetailsSectionController detailsSectionController(EventPublisher eventPublisher,
                                                             ViewLoader viewLoader,
                                                             TaskExecutor taskExecutor) {
        return new DetailsSectionController(eventPublisher, viewLoader, taskExecutor);
    }

    @Bean
    public ViewLoader viewLoader(ApplicationContext applicationContext, ViewManager viewManager, LocaleText localeText, OptionsService optionsService) {
        return PopcornViewLoader.builder()
                .applicationContext(applicationContext)
                .viewManager(viewManager)
                .localeText(localeText)
                .optionsService(optionsService)
                .build();
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
    @ConditionalOnTvMode
    public TvFilterComponent tvFilterComponent(EventPublisher eventPublisher,
                                               LocaleText localeText,
                                               FxLib fxLib,
                                               PopcornFx instance) {
        return new TvFilterComponent(eventPublisher, localeText, fxLib, instance);
    }

    @Bean
    @ConditionalOnDesktopMode
    public WindowComponent windowComponent(MaximizeService maximizeService,
                                           PlatformProvider platformProvider) {
        return new WindowComponent(maximizeService, platformProvider);
    }

    @Bean
    @ConditionalOnTvMode
    public SystemTimeComponent systemTimeComponent() {
        return new SystemTimeComponent();
    }

    @Bean
    @ConditionalOnDesktopMode
    public DesktopPosterComponent desktopPosterComponent(EventPublisher eventPublisher,
                                                         ImageService imageService,
                                                         FavoriteService favoriteService,
                                                         WatchedService watchedService,
                                                         LocaleText localeText) {
        return new DesktopPosterComponent(eventPublisher, imageService, favoriteService, watchedService, localeText);
    }

    @Bean
    @ConditionalOnTvMode
    public TvPosterComponent tvPosterComponent(EventPublisher eventPublisher,
                                               ImageService imageService) {
        return new TvPosterComponent(eventPublisher, imageService);
    }

    @Bean
    @ConditionalOnDesktopMode
    public DesktopMovieActionsComponent desktopMovieActionsComponent(PlayerManagerService playerService,
                                                                     EventPublisher eventPublisher) {
        return new DesktopMovieActionsComponent(playerService, eventPublisher);
    }

    @Bean
    @ConditionalOnTvMode
    public TvMovieActionsComponent tvMovieActionsComponent(EventPublisher eventPublisher) {
        return new TvMovieActionsComponent(eventPublisher);
    }

    @Bean
    @ConditionalOnDesktopMode
    public DesktopSidebarSearchComponent desktopSidebarSearchComponent(EventPublisher eventPublisher) {
        return new DesktopSidebarSearchComponent(eventPublisher);
    }

    @Bean
    @ConditionalOnTvMode
    public TvSidebarSearchComponent tvSidebarSearchComponent(EventPublisher eventPublisher) {
        return new TvSidebarSearchComponent(eventPublisher);
    }
}
