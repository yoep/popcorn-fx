package com.github.yoep.popcorn.ui.view.config;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.ui.activities.ActivityManager;
import com.github.yoep.popcorn.ui.config.properties.PopcornProperties;
import com.github.yoep.popcorn.ui.media.favorites.FavoriteService;
import com.github.yoep.popcorn.ui.media.providers.ProviderService;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.media.watched.WatchedService;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.view.conditions.ConditionalOnTvMode;
import com.github.yoep.popcorn.ui.view.controllers.MainController;
import com.github.yoep.popcorn.ui.view.controllers.tv.MainTvController;
import com.github.yoep.popcorn.ui.view.controllers.tv.components.*;
import com.github.yoep.popcorn.ui.view.controllers.tv.sections.*;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import com.github.yoep.popcorn.ui.view.services.UrlService;
import com.github.yoep.popcorn.ui.view.services.VideoPlayerService;
import com.github.yoep.torrent.adapter.TorrentService;
import com.github.yoep.torrent.adapter.TorrentStreamService;
import org.springframework.boot.ApplicationArguments;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.task.TaskExecutor;

import java.util.List;

@Configuration
@ConditionalOnTvMode
public class TvConfig {
    @Bean
    public MainController mainController(ActivityManager activityManager, ViewLoader viewLoader, ApplicationArguments arguments, UrlService urlService,
                                         SettingsService settingsService, TaskExecutor taskExecutor) {
        return MainTvController.builder()
                .activityManager(activityManager)
                .viewLoader(viewLoader)
                .arguments(arguments)
                .urlService(urlService)
                .settingsService(settingsService)
                .taskExecutor(taskExecutor)
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
    public ListSectionController listSectionController(ActivityManager activityManager,
                                                       List<ProviderService<? extends Media>> providerServices,
                                                       ViewLoader viewLoader,
                                                       LocaleText localeText,
                                                       WatchedService watchedService,
                                                       ImageService imageService) {
        return new ListSectionController(activityManager, providerServices, viewLoader, localeText, watchedService, imageService);
    }

    @Bean
    public LoaderSectionController loaderSectionController() {
        return new LoaderSectionController();
    }

    @Bean
    public MenuSectionController menuSectionController(ActivityManager activityManager,
                                                       SettingsService settingsService,
                                                       PopcornProperties properties) {
        return new MenuSectionController(activityManager, settingsService, properties);
    }

    @Bean
    public PlayerSectionController playerSectionController(ActivityManager activityManager,
                                                           SettingsService settingsService,
                                                           VideoPlayerService videoPlayerService,
                                                           LocaleText localeText) {
        return new PlayerSectionController(activityManager, settingsService, videoPlayerService, localeText);
    }

    @Bean
    public SettingsSectionController settingsSectionController() {
        return new SettingsSectionController();
    }

    //endregion

    //region Components

    @Bean
    public MovieDetailsComponent movieDetailsComponent(LocaleText localeText,
                                                       ActivityManager activityManager,
                                                       SubtitleService subtitleService,
                                                       FavoriteService favoriteService,
                                                       TorrentService torrentService,
                                                       ImageService imageService,
                                                       SettingsService settingsService) {
        return new MovieDetailsComponent(localeText, activityManager, subtitleService, favoriteService, torrentService, imageService, settingsService);
    }

    @Bean
    public ShowDetailsComponent showDetailsComponent(LocaleText localeText,
                                                     ActivityManager activityManager,
                                                     TorrentService torrentService,
                                                     ImageService imageService,
                                                     SettingsService settingsService) {
        return new ShowDetailsComponent(localeText, activityManager, torrentService, imageService, settingsService);
    }

    @Bean
    public SettingsUiComponent settingsUiComponent(ActivityManager activityManager,
                                                   LocaleText localeText,
                                                   SettingsService settingsService) {
        return new SettingsUiComponent(activityManager, localeText, settingsService);
    }

    @Bean
    public SettingsPlaybackComponent settingsPlaybackComponent(ActivityManager activityManager,
                                                               LocaleText localeText,
                                                               SettingsService settingsService) {
        return new SettingsPlaybackComponent(activityManager, localeText, settingsService);
    }

    @Bean
    public SettingsSubtitlesComponent settingsSubtitlesComponent(ActivityManager activityManager,
                                                                 LocaleText localeText,
                                                                 SettingsService settingsService) {
        return new SettingsSubtitlesComponent(activityManager, localeText, settingsService);
    }

    @Bean
    public PlayerHeaderComponent playerHeaderComponent(ActivityManager activityManager) {
        return new PlayerHeaderComponent(activityManager);
    }

    @Bean
    public PlayerControlsComponent playerControlsComponent(ActivityManager activityManager,
                                                           VideoPlayerService videoPlayerService) {
        return new PlayerControlsComponent(activityManager, videoPlayerService);
    }

    @Bean
    public LoaderTorrentComponent loaderTorrentComponent(LocaleText localeText,
                                                         TorrentService torrentService,
                                                         TorrentStreamService torrentStreamService,
                                                         ActivityManager activityManager,
                                                         TaskExecutor taskExecutor,
                                                         SubtitleService subtitleService,
                                                         ImageService imageService,
                                                         SettingsService settingsService) {
        return new LoaderTorrentComponent(localeText, torrentService, torrentStreamService, activityManager, taskExecutor, subtitleService, imageService, settingsService);
    }

    //endregion
}
