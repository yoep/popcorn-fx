package com.github.yoep.popcorn.ui.view.config;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.ui.config.properties.PopcornProperties;
import com.github.yoep.popcorn.ui.events.ActivityManager;
import com.github.yoep.popcorn.ui.media.favorites.FavoriteService;
import com.github.yoep.popcorn.ui.media.providers.ProviderService;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.media.providers.models.Movie;
import com.github.yoep.popcorn.ui.media.providers.models.Show;
import com.github.yoep.popcorn.ui.media.watched.WatchedService;
import com.github.yoep.popcorn.ui.settings.OptionsService;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.subtitles.SubtitlePickerService;
import com.github.yoep.popcorn.ui.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.torrent.TorrentCollectionService;
import com.github.yoep.popcorn.ui.trakt.TraktService;
import com.github.yoep.popcorn.ui.view.conditions.ConditionalOnDesktopMode;
import com.github.yoep.popcorn.ui.view.controllers.MainController;
import com.github.yoep.popcorn.ui.view.controllers.desktop.MainDesktopController;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.*;
import com.github.yoep.popcorn.ui.view.controllers.desktop.sections.*;
import com.github.yoep.popcorn.ui.view.services.*;
import com.github.yoep.torrent.adapter.TorrentService;
import com.github.yoep.torrent.adapter.TorrentStreamService;
import org.springframework.boot.ApplicationArguments;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.task.TaskExecutor;

import java.util.List;

@Configuration
@ConditionalOnDesktopMode
public class DesktopConfig {
    @Bean
    public MainController mainController(ViewLoader viewLoader,
                                         TaskExecutor taskExecutor,
                                         ApplicationArguments arguments,
                                         UrlService urlService,
                                         SettingsService settingsService) {
        return MainDesktopController.builder()
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
    public DetailsSectionController detailsSectionController(ActivityManager activityManager,
                                                             ViewLoader viewLoader,
                                                             TaskExecutor taskExecutor) {
        return new DetailsSectionController(activityManager, viewLoader, taskExecutor);
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
                                                         TorrentStreamService torrentStreamService,
                                                         SubtitleService subtitleService,
                                                         LocaleText localeText,
                                                         ImageService imageService,
                                                         SettingsService settingsService) {
        return new LoaderTorrentComponent(activityManager, taskExecutor, torrentService, torrentStreamService, subtitleService, localeText, imageService, settingsService);
    }

    @Bean
    public LoaderUrlComponent loaderUrlComponent(LocaleText localeText,
                                                 TorrentService torrentService,
                                                 TorrentStreamService torrentStreamService,
                                                 ActivityManager activityManager,
                                                 TaskExecutor taskExecutor) {
        return new LoaderUrlComponent(localeText, torrentService, torrentStreamService, activityManager, taskExecutor);
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
                                                       SubtitlePickerService subtitlePickerService,
                                                       FavoriteService favoriteService,
                                                       WatchedService watchedService,
                                                       ImageService imageService,
                                                       SettingsService settingsService) {
        return new MovieDetailsComponent(activityManager, localeText, torrentService, subtitleService, subtitlePickerService, favoriteService, watchedService, imageService, settingsService);
    }

    @Bean
    public ShowDetailsComponent showDetailsComponent(ActivityManager activityManager,
                                                     LocaleText localeText,
                                                     TorrentService torrentService,
                                                     SubtitleService subtitleService,
                                                     SubtitlePickerService subtitlePickerService,
                                                     FavoriteService favoriteService,
                                                     WatchedService watchedService,
                                                     ImageService imageService,
                                                     SettingsService settingsService) {
        return new ShowDetailsComponent(activityManager, localeText, torrentService, subtitleService, subtitlePickerService, favoriteService, watchedService, imageService, settingsService);
    }

    @Bean
    public PlayerHeaderComponent playerHeaderComponent(ActivityManager activityManager,
                                                       VideoPlayerService videoPlayerService,
                                                       LocaleText localeText) {
        return new PlayerHeaderComponent(activityManager, videoPlayerService, localeText);
    }

    @Bean
    public PlayerControlsComponent playerControlsComponent(ActivityManager activityManager,
                                                           VideoPlayerService videoPlayerService,
                                                           SubtitleService subtitleService,
                                                           LocaleText localeText) {
        return new PlayerControlsComponent(activityManager, videoPlayerService, subtitleService, localeText);
    }

    @Bean
    public SettingsSubtitlesComponent settingsSubtitlesComponent(SettingsService settingsService, LocaleText localeText) {
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
    public SettingsPlaybackComponent settingsPlaybackComponent(ActivityManager activityManager,
                                                               LocaleText localeText,
                                                               SettingsService settingsService) {
        return new SettingsPlaybackComponent(activityManager, localeText, settingsService);
    }

    @Bean
    public TitleBarComponent titleBarComponent(MaximizeService maximizeService,
                                               OptionsService optionsService) {
        return new TitleBarComponent(maximizeService, optionsService);
    }

    @Bean
    public PlayerPlayNextComponent playerPlaylistComponent(ImageService imageService,
                                                           PlayNextService playNextService) {
        return new PlayerPlayNextComponent(imageService, playNextService);
    }

    //endregion
}
