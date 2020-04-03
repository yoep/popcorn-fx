package com.github.yoep.popcorn.view.config;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.config.properties.PopcornProperties;
import com.github.yoep.popcorn.media.providers.ProviderService;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.watched.WatchedService;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.subtitles.SubtitleService;
import com.github.yoep.popcorn.torrent.TorrentService;
import com.github.yoep.popcorn.view.conditions.ConditionalOnTvMode;
import com.github.yoep.popcorn.view.controllers.MainController;
import com.github.yoep.popcorn.view.controllers.tv.MainTvController;
import com.github.yoep.popcorn.view.controllers.tv.components.*;
import com.github.yoep.popcorn.view.controllers.tv.sections.*;
import com.github.yoep.popcorn.view.services.ImageService;
import com.github.yoep.popcorn.view.services.UrlService;
import com.github.yoep.popcorn.view.services.VideoPlayerService;
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
    public MovieDetailsComponent movieDetailsComponent(ActivityManager activityManager,
                                                       TorrentService torrentService,
                                                       ImageService imageService) {
        return new MovieDetailsComponent(activityManager, torrentService, imageService);
    }

    @Bean
    public ShowDetailsComponent showDetailsComponent(ActivityManager activityManager,
                                                     TorrentService torrentService,
                                                     ImageService imageService) {
        return new ShowDetailsComponent(activityManager, torrentService, imageService);
    }

    @Bean
    public SettingsUiComponent settingsUiComponent() {
        return new SettingsUiComponent();
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
                                                         ActivityManager activityManager,
                                                         TaskExecutor taskExecutor,
                                                         SubtitleService subtitleService,
                                                         ImageService imageService) {
        return new LoaderTorrentComponent(localeText, torrentService, activityManager, taskExecutor, subtitleService, imageService);
    }

    //endregion
}
