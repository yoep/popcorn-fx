package com.github.yoep.popcorn.ui.view.config;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.player.adapter.PlayerService;
import com.github.yoep.popcorn.ui.config.properties.PopcornProperties;
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
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.task.TaskExecutor;

import java.util.List;

@Configuration
@ConditionalOnDesktopMode
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
    public HeaderSectionController headerSectionController(ApplicationEventPublisher eventPublisher,
                                                           PopcornProperties properties,
                                                           LocaleText localeText,
                                                           SettingsService settingsService) {
        return new HeaderSectionController(eventPublisher, properties, localeText, settingsService);
    }

    @Bean
    public ListSectionController listSectionController(List<ProviderService<? extends Media>> providerServices,
                                                       FavoriteService favoriteService,
                                                       WatchedService watchedService,
                                                       ViewLoader viewLoader,
                                                       LocaleText localeText,
                                                       ImageService imageService) {
        return new ListSectionController(providerServices, favoriteService, watchedService, viewLoader, localeText, imageService);
    }

    @Bean
    public LoaderSectionController loaderSectionController(ViewLoader viewLoader, TaskExecutor taskExecutor) {
        return new LoaderSectionController(viewLoader, taskExecutor);
    }

    @Bean
    public PlayerSectionController playerSectionController(SettingsService settingsService,
                                                           VideoPlayerService videoPlayerService,
                                                           VideoPlayerManagerService videoPlayerManagerService,
                                                           VideoPlayerSubtitleService videoPlayerSubtitleService,
                                                           LocaleText localeText) {
        return new PlayerSectionController(settingsService, videoPlayerService, videoPlayerManagerService, videoPlayerSubtitleService, localeText);
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
    public LoaderTorrentComponent loaderTorrentComponent(LocaleText localeText,
                                                         TorrentService torrentService,
                                                         TorrentStreamService torrentStreamService,
                                                         ApplicationEventPublisher eventPublisher,
                                                         ImageService imageService,
                                                         SubtitleService subtitleService,
                                                         TaskExecutor taskExecutor,
                                                         SettingsService settingsService) {
        return new LoaderTorrentComponent(localeText, torrentService, torrentStreamService, eventPublisher, imageService, subtitleService, taskExecutor,
                settingsService);
    }

    @Bean
    public LoaderUrlComponent loaderUrlComponent(LocaleText localeText,
                                                 TorrentService torrentService,
                                                 TorrentStreamService torrentStreamService,
                                                 ApplicationEventPublisher eventPublisher,
                                                 TaskExecutor taskExecutor) {
        return new LoaderUrlComponent(localeText, torrentService, torrentStreamService, eventPublisher, taskExecutor);
    }

    @Bean
    public SettingsUIComponent settingsUIComponent(ApplicationEventPublisher eventPublisher,
                                                   LocaleText localeText,
                                                   SettingsService settingsService) {
        return new SettingsUIComponent(eventPublisher, localeText, settingsService);
    }

    @Bean
    public MovieDetailsComponent movieDetailsComponent(ApplicationEventPublisher eventPublisher,
                                                       LocaleText localeText,
                                                       HealthService healthService,
                                                       SubtitleService subtitleService,
                                                       SubtitlePickerService subtitlePickerService,
                                                       ImageService imageService,
                                                       SettingsService settingsService,
                                                       FavoriteService favoriteService,
                                                       WatchedService watchedService,
                                                       PlayerService playerService) {
        return new MovieDetailsComponent(eventPublisher, localeText, healthService, subtitleService, subtitlePickerService, imageService, settingsService,
                favoriteService, watchedService, playerService);
    }

    @Bean
    public ShowDetailsComponent showDetailsComponent(ApplicationEventPublisher eventPublisher,
                                                     LocaleText localeText,
                                                     HealthService healthService,
                                                     SubtitleService subtitleService,
                                                     SubtitlePickerService subtitlePickerService,
                                                     ImageService imageService,
                                                     SettingsService settingsService,
                                                     FavoriteService favoriteService,
                                                     WatchedService watchedService,
                                                     ShowHelperService showHelperService) {
        return new ShowDetailsComponent(eventPublisher, localeText, healthService, subtitleService, subtitlePickerService, imageService, settingsService,
                favoriteService, watchedService, showHelperService);
    }

    @Bean
    public PlayerHeaderComponent playerHeaderComponent(VideoPlayerService videoPlayerService,
                                                       LocaleText localeText) {
        return new PlayerHeaderComponent(videoPlayerService, localeText);
    }

    @Bean
    public PlayerControlsComponent playerControlsComponent(VideoPlayerService videoPlayerService,
                                                           VideoPlayerManagerService videoPlayerManagerService,
                                                           VideoPlayerSubtitleService videoPlayerSubtitleService,
                                                           SubtitleService subtitleService,
                                                           LocaleText localeText) {
        return new PlayerControlsComponent(videoPlayerService, videoPlayerManagerService, videoPlayerSubtitleService, subtitleService, localeText);
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
