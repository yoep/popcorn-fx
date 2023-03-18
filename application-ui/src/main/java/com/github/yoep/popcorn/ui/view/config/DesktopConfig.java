package com.github.yoep.popcorn.ui.view.config;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.providers.ProviderService;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.MovieOverview;
import com.github.yoep.popcorn.backend.media.providers.models.ShowOverview;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.OptionsService;
import com.github.yoep.popcorn.ui.playnext.PlayNextService;
import com.github.yoep.popcorn.ui.torrent.TorrentCollectionService;
import com.github.yoep.popcorn.ui.trakt.TraktService;
import com.github.yoep.popcorn.ui.view.controllers.common.sections.ListSectionController;
import com.github.yoep.popcorn.ui.view.controllers.common.sections.LoaderSectionController;
import com.github.yoep.popcorn.ui.view.controllers.common.sections.SettingsSectionController;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.*;
import com.github.yoep.popcorn.ui.view.controllers.desktop.sections.TorrentCollectionSectionController;
import com.github.yoep.popcorn.ui.view.controllers.desktop.sections.WatchlistSectionController;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.popcorn.ui.view.services.TorrentSettingService;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.task.TaskExecutor;

import java.util.List;

@Configuration
public class DesktopConfig {

    //region Sections

    @Bean
    public ListSectionController listSectionController(List<ProviderService<? extends Media>> providerServices,
                                                       FavoriteService favoriteService,
                                                       WatchedService watchedService,
                                                       ViewLoader viewLoader,
                                                       LocaleText localeText,
                                                       EventPublisher eventPublisher,
                                                       ImageService imageService,
                                                       OptionsService optionsService) {
        return new ListSectionController(providerServices, favoriteService, watchedService, viewLoader, localeText, eventPublisher, imageService,
                optionsService);
    }

    @Bean
    public LoaderSectionController loaderSectionController(ViewLoader viewLoader, TaskExecutor taskExecutor, EventPublisher eventPublisher) {
        return new LoaderSectionController(viewLoader, taskExecutor, eventPublisher);
    }

    @Bean
    public SettingsSectionController settingsSectionController(EventPublisher eventPublisher) {
        return new SettingsSectionController(eventPublisher);
    }

    @Bean
    public TorrentCollectionSectionController torrentCollectionSectionController(EventPublisher eventPublisher,
                                                                                 TorrentCollectionService torrentCollectionService,
                                                                                 LocaleText localeText) {
        return new TorrentCollectionSectionController(eventPublisher, torrentCollectionService, localeText);
    }

    @Bean
    public WatchlistSectionController watchlistSectionController(EventPublisher eventPublisher,
                                                                 ViewLoader viewLoader,
                                                                 LocaleText localeText,
                                                                 TraktService traktService,
                                                                 ProviderService<MovieOverview> movieProviderService,
                                                                 ProviderService<ShowOverview> showProviderService,
                                                                 ImageService imageService) {
        return new WatchlistSectionController(eventPublisher, viewLoader, localeText, traktService, movieProviderService, showProviderService, imageService);
    }

    //endregion

    //region Components

    @Bean
    public SettingsUIComponent settingsUIComponent(EventPublisher eventPublisher,
                                                   LocaleText localeText,
                                                   ApplicationConfig settingsService) {
        return new SettingsUIComponent(eventPublisher, localeText, settingsService);
    }

    @Bean
    public SettingsSubtitlesComponent settingsSubtitlesComponent(ApplicationConfig settingsService, LocaleText localeText) {
        return new SettingsSubtitlesComponent(settingsService, localeText);
    }

    @Bean
    public SettingsTorrentComponent settingsTorrentComponent(EventPublisher eventPublisher,
                                                             LocaleText localeText,
                                                             ApplicationConfig settingsService,
                                                             TorrentSettingService torrentSettingService) {
        return new SettingsTorrentComponent(eventPublisher, localeText, settingsService, torrentSettingService);
    }

    @Bean
    public SettingsTraktComponent settingsTraktComponent(TraktService traktService) {
        return new SettingsTraktComponent(traktService);
    }

    @Bean
    public SettingsPlaybackComponent settingsPlaybackComponent(EventPublisher eventPublisher,
                                                               LocaleText localeText,
                                                               ApplicationConfig settingsService) {
        return new SettingsPlaybackComponent(eventPublisher, localeText, settingsService);
    }

    @Bean
    public SettingsServerComponent settingsServerComponent(EventPublisher eventPublisher,
                                                           LocaleText localeText,
                                                           ApplicationConfig settingsService) {
        return new SettingsServerComponent(eventPublisher, localeText, settingsService);
    }

    @Bean
    public PlayerPlayNextComponent playerPlaylistComponent(ImageService imageService,
                                                           PlayNextService playNextService) {
        return new PlayerPlayNextComponent(imageService, playNextService);
    }

    //endregion
}
