package com.github.yoep.popcorn.view.config;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.config.properties.PopcornProperties;
import com.github.yoep.popcorn.media.favorites.FavoriteService;
import com.github.yoep.popcorn.media.providers.ProviderService;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Movie;
import com.github.yoep.popcorn.media.providers.models.Show;
import com.github.yoep.popcorn.media.watched.WatchedService;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.subtitles.SubtitleService;
import com.github.yoep.popcorn.torrent.TorrentCollectionService;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.popcorn.trakt.TraktService;
import com.github.yoep.popcorn.view.conditions.ConditionalOnDesktopMode;
import com.github.yoep.popcorn.view.controllers.MainController;
import com.github.yoep.popcorn.view.controllers.desktop.MainDesktopController;
import com.github.yoep.popcorn.view.controllers.desktop.components.*;
import com.github.yoep.popcorn.view.controllers.desktop.sections.*;
import com.github.yoep.popcorn.view.services.ImageService;
import com.github.yoep.popcorn.view.services.MaximizeService;
import com.github.yoep.popcorn.view.services.UrlService;
import com.github.yoep.popcorn.view.services.VideoPlayerService;
import org.springframework.boot.ApplicationArguments;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.task.TaskExecutor;

import java.util.List;

@Configuration
@ConditionalOnDesktopMode
public class DesktopConfig {
    @Bean
    public MainController mainController(ActivityManager activityManager,
                                         ViewLoader viewLoader,
                                         TaskExecutor taskExecutor,
                                         ApplicationArguments arguments,
                                         UrlService urlService,
                                         SettingsService settingsService) {
        return MainDesktopController.builder()
                .activityManager(activityManager)
                .arguments(arguments)
                .taskExecutor(taskExecutor)
                .viewLoader(viewLoader)
                .urlService(urlService)
                .settingsService(settingsService)
                .build();
    }

    //region Sections

    @Bean
    public ContentSectionController contentSectionController(ActivityManager activityManager, ViewLoader viewLoader, TaskExecutor taskExecutor) {
        return new ContentSectionController(activityManager, viewLoader, taskExecutor);
    }

    @Bean
    public DetailsSectionController detailsSectionController(ActivityManager activityManager) {
        return new DetailsSectionController(activityManager);
    }

    @Bean
    public HeaderSectionController headerSectionController(ActivityManager activityManager, PopcornProperties properties, LocaleText localeText,
                                                           SettingsService settingsService) {
        return new HeaderSectionController(activityManager, properties, localeText, settingsService);
    }

    @Bean
    public ListSectionController listSectionController(ActivityManager activityManager,
                                                       List<ProviderService<? extends Media>> providerServices,
                                                       FavoriteService favoriteService,
                                                       WatchedService watchedService,
                                                       ViewLoader viewLoader,
                                                       LocaleText localeText,
                                                       ImageService imageService) {
        return new ListSectionController(activityManager, providerServices, favoriteService, watchedService, viewLoader, localeText, imageService);
    }

    @Bean
    public LoaderSectionController loaderSectionController(ActivityManager activityManager, ViewLoader viewLoader, TaskExecutor taskExecutor) {
        return new LoaderSectionController(activityManager, viewLoader, taskExecutor);
    }

    @Bean
    public OverlaySectionController overlaySectionController(ActivityManager activityManager, ViewLoader viewLoader, TaskExecutor taskExecutor) {
        return new OverlaySectionController(activityManager, viewLoader, taskExecutor);
    }

    @Bean
    public PlayerSectionController playerSectionController(ActivityManager activityManager,
                                                           SettingsService settingsService,
                                                           VideoPlayerService videoPlayerService,
                                                           LocaleText localeText) {
        return new PlayerSectionController(activityManager, settingsService, videoPlayerService, localeText);
    }

    @Bean
    public SettingsSectionController settingsSectionController(ActivityManager activityManager) {
        return new SettingsSectionController(activityManager);
    }

    @Bean
    public TorrentCollectionSectionController torrentCollectionSectionController(TorrentCollectionService torrentCollectionService,
                                                                                 ActivityManager activityManager,
                                                                                 LocaleText localeText) {
        return new TorrentCollectionSectionController(torrentCollectionService, activityManager, localeText);
    }

    @Bean
    public WatchlistSectionController watchlistSectionController(ActivityManager activityManager,
                                                                 ViewLoader viewLoader,
                                                                 LocaleText localeText,
                                                                 TraktService traktService,
                                                                 ProviderService<Movie> movieProviderService,
                                                                 ProviderService<Show> showProviderService,
                                                                 ImageService imageService) {
        return WatchlistSectionController.builder()
                .activityManager(activityManager)
                .viewLoader(viewLoader)
                .localeText(localeText)
                .traktService(traktService)
                .movieProviderService(movieProviderService)
                .showProviderService(showProviderService)
                .imageService(imageService)
                .build();
    }

    //endregion

    //region Components

    @Bean
    public DetailsTorrentComponent detailsTorrentComponent(ActivityManager activityManager,
                                                           TorrentCollectionService torrentCollectionService,
                                                           LocaleText localeText) {
        return new DetailsTorrentComponent(activityManager, torrentCollectionService, localeText);
    }

    @Bean
    public LoaderTorrentComponent loaderTorrentComponent(ActivityManager activityManager,
                                                         TaskExecutor taskExecutor,
                                                         TorrentService torrentService,
                                                         SubtitleService subtitleService,
                                                         LocaleText localeText,
                                                         ImageService imageService) {
        return new LoaderTorrentComponent(activityManager, taskExecutor, torrentService, subtitleService, localeText, imageService);
    }

    @Bean
    public LoaderUrlComponent loaderUrlComponent(LocaleText localeText,
                                                 TorrentService torrentService,
                                                 ActivityManager activityManager,
                                                 TaskExecutor taskExecutor) {
        return new LoaderUrlComponent(localeText, torrentService, activityManager, taskExecutor);
    }

    @Bean
    public SettingsUIComponent settingsUIComponent(ActivityManager activityManager,
                                                   LocaleText localeText,
                                                   SettingsService settingsService) {
        return new SettingsUIComponent(activityManager, localeText, settingsService);
    }

    @Bean
    public MovieDetailsComponent movieDetailsComponent(ActivityManager activityManager,
                                                       LocaleText localeText,
                                                       TorrentService torrentService,
                                                       SubtitleService subtitleService,
                                                       FavoriteService favoriteService,
                                                       WatchedService watchedService,
                                                       ImageService imageService) {
        return new MovieDetailsComponent(activityManager, localeText, torrentService, subtitleService, favoriteService, watchedService, imageService);
    }

    @Bean
    public ShowDetailsComponent showDetailsComponent(ActivityManager activityManager,
                                                     LocaleText localeText,
                                                     TorrentService torrentService,
                                                     SubtitleService subtitleService,
                                                     FavoriteService favoriteService,
                                                     WatchedService watchedService,
                                                     ImageService imageService) {
        return new ShowDetailsComponent(activityManager, localeText, torrentService, subtitleService, favoriteService, watchedService, imageService);
    }

    @Bean
    public PlayerHeaderComponent playerHeaderComponent(ActivityManager activityManager,
                                                       TorrentService torrentService,
                                                       VideoPlayerService videoPlayerService,
                                                       LocaleText localeText) {
        return new PlayerHeaderComponent(activityManager, torrentService, videoPlayerService, localeText);
    }

    @Bean
    public PlayerControlsComponent playerControlsComponent(ActivityManager activityManager,
                                                           VideoPlayerService videoPlayerService,
                                                           SubtitleService subtitleService,
                                                           LocaleText localeText) {
        return new PlayerControlsComponent(activityManager, videoPlayerService, subtitleService, localeText);
    }

    @Bean
    public SettingsSubtitlesComponent settingsSubtitlesComponent(SettingsService settingsService,  LocaleText localeText) {
        return new SettingsSubtitlesComponent(settingsService, localeText);
    }

    @Bean
    public SettingsTorrentComponent settingsTorrentComponent(ActivityManager activityManager,
                                                             LocaleText localeText,
                                                             SettingsService settingsService) {
        return new SettingsTorrentComponent(activityManager, localeText, settingsService);
    }

    @Bean
    public SettingsTraktComponent settingsTraktComponent(TraktService traktService) {
        return new SettingsTraktComponent(traktService);
    }

    @Bean
    public TitleBarComponent titleBarComponent(MaximizeService maximizeService) {
        return new TitleBarComponent(maximizeService);
    }

    //endregion
}
