package com.github.yoep.popcorn.ui.view.config;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.providers.ProviderService;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.Show;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.backend.settings.OptionsService;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.ui.playnext.PlayNextService;
import com.github.yoep.popcorn.ui.torrent.TorrentCollectionService;
import com.github.yoep.popcorn.ui.trakt.TraktService;
import com.github.yoep.popcorn.ui.view.controllers.MainController;
import com.github.yoep.popcorn.ui.view.controllers.desktop.MainDesktopController;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.*;
import com.github.yoep.popcorn.ui.view.controllers.desktop.sections.*;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.popcorn.ui.view.services.TorrentSettingService;
import com.github.yoep.popcorn.ui.view.services.UrlService;
import org.springframework.boot.ApplicationArguments;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.task.TaskExecutor;

import java.util.List;

@Configuration
public class DesktopConfig {

    @Bean
    public MainController mainController(ApplicationEventPublisher eventPublisher,
                                         ViewLoader viewLoader,
                                         TaskExecutor taskExecutor,
                                         ApplicationArguments arguments,
                                         UrlService urlService,
                                         SettingsService settingsService,
                                         OptionsService optionsService) {
        return new MainDesktopController(eventPublisher, viewLoader, taskExecutor, arguments, urlService, settingsService, optionsService);
    }

    //region Sections

    @Bean
    public ContentSectionController contentSectionController(ViewLoader viewLoader,
                                                             TaskExecutor taskExecutor,
                                                             LocaleText localeText,
                                                             ApplicationEventPublisher eventPublisher) {
        return new ContentSectionController(viewLoader, taskExecutor, localeText, eventPublisher);
    }

    @Bean
    public DetailsSectionController detailsSectionController(ApplicationEventPublisher eventPublisher,
                                                             ViewLoader viewLoader,
                                                             TaskExecutor taskExecutor) {
        return new DetailsSectionController(eventPublisher, viewLoader, taskExecutor);
    }

    @Bean
    public ListSectionController listSectionController(List<ProviderService<? extends Media>> providerServices,
                                                       FavoriteService favoriteService,
                                                       WatchedService watchedService,
                                                       ViewLoader viewLoader,
                                                       LocaleText localeText,
                                                       ApplicationEventPublisher eventPublisher,
                                                       ImageService imageService) {
        return new ListSectionController(providerServices, favoriteService, watchedService, viewLoader, localeText, eventPublisher, imageService);
    }

    @Bean
    public LoaderSectionController loaderSectionController(ViewLoader viewLoader, TaskExecutor taskExecutor) {
        return new LoaderSectionController(viewLoader, taskExecutor);
    }

    @Bean
    public SettingsSectionController settingsSectionController(ApplicationEventPublisher eventPublisher) {
        return new SettingsSectionController(eventPublisher);
    }

    @Bean
    public TorrentCollectionSectionController torrentCollectionSectionController(ApplicationEventPublisher eventPublisher,
                                                                                 TorrentCollectionService torrentCollectionService,
                                                                                 LocaleText localeText) {
        return new TorrentCollectionSectionController(eventPublisher, torrentCollectionService, localeText);
    }

    @Bean
    public WatchlistSectionController watchlistSectionController(ApplicationEventPublisher eventPublisher,
                                                                 ViewLoader viewLoader,
                                                                 LocaleText localeText,
                                                                 TraktService traktService,
                                                                 ProviderService<Movie> movieProviderService,
                                                                 ProviderService<Show> showProviderService,
                                                                 ImageService imageService) {
        return new WatchlistSectionController(eventPublisher, viewLoader, localeText, traktService, movieProviderService, showProviderService, imageService);
    }

    //endregion

    //region Components

    @Bean
    public DetailsTorrentComponent detailsTorrentComponent(ApplicationEventPublisher eventPublisher,
                                                           TorrentCollectionService torrentCollectionService,
                                                           LocaleText localeText) {
        return new DetailsTorrentComponent(eventPublisher, torrentCollectionService, localeText);
    }

    @Bean
    public SettingsUIComponent settingsUIComponent(ApplicationEventPublisher eventPublisher,
                                                   LocaleText localeText,
                                                   SettingsService settingsService) {
        return new SettingsUIComponent(eventPublisher, localeText, settingsService);
    }

    @Bean
    public SettingsSubtitlesComponent settingsSubtitlesComponent(SettingsService settingsService, LocaleText localeText) {
        return new SettingsSubtitlesComponent(settingsService, localeText);
    }

    @Bean
    public SettingsTorrentComponent settingsTorrentComponent(ApplicationEventPublisher eventPublisher,
                                                             LocaleText localeText,
                                                             SettingsService settingsService,
                                                             TorrentSettingService torrentSettingService) {
        return new SettingsTorrentComponent(eventPublisher, localeText, settingsService, torrentSettingService);
    }

    @Bean
    public SettingsTraktComponent settingsTraktComponent(TraktService traktService) {
        return new SettingsTraktComponent(traktService);
    }

    @Bean
    public SettingsPlaybackComponent settingsPlaybackComponent(ApplicationEventPublisher eventPublisher,
                                                               LocaleText localeText,
                                                               SettingsService settingsService) {
        return new SettingsPlaybackComponent(eventPublisher, localeText, settingsService);
    }

    @Bean
    public SettingsServerComponent settingsServerComponent(ApplicationEventPublisher eventPublisher,
                                                           LocaleText localeText,
                                                           SettingsService settingsService) {
        return new SettingsServerComponent(eventPublisher, localeText, settingsService);
    }

    @Bean
    public PlayerPlayNextComponent playerPlaylistComponent(ImageService imageService,
                                                           PlayNextService playNextService) {
        return new PlayerPlayNextComponent(imageService, playNextService);
    }

    //endregion
}
