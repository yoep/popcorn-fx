package com.github.yoep.popcorn.ui.view.config;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.ui.config.properties.PopcornProperties;
import com.github.yoep.popcorn.ui.media.favorites.FavoriteService;
import com.github.yoep.popcorn.ui.media.providers.ProviderService;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.media.watched.WatchedService;
import com.github.yoep.popcorn.ui.settings.OptionsService;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.view.conditions.ConditionalOnTvMode;
import com.github.yoep.popcorn.ui.view.controllers.MainController;
import com.github.yoep.popcorn.ui.view.controllers.tv.MainTvController;
import com.github.yoep.popcorn.ui.view.controllers.tv.components.*;
import com.github.yoep.popcorn.ui.view.controllers.tv.sections.*;
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
@ConditionalOnTvMode
public class TvConfig {
    @Bean
    public MainController mainController(ApplicationEventPublisher eventPublisher,
                                         ViewLoader viewLoader,
                                         ApplicationArguments arguments,
                                         UrlService urlService,
                                         SettingsService settingsService,
                                         OptionsService optionsService,
                                         TaskExecutor taskExecutor) {
        return new MainTvController(eventPublisher, viewLoader, arguments, urlService, settingsService, optionsService, taskExecutor);
    }

    //region Sections

    @Bean
    public ContentSectionController contentSectionController(ViewLoader viewLoader, TaskExecutor taskExecutor) {
        return new ContentSectionController(viewLoader, taskExecutor);
    }

    @Bean
    public DetailsSectionController detailsSectionController() {
        return new DetailsSectionController();
    }

    @Bean
    public ListSectionController listSectionController(List<ProviderService<? extends Media>> providerServices,
                                                       ViewLoader viewLoader,
                                                       LocaleText localeText,
                                                       WatchedService watchedService,
                                                       ImageService imageService) {
        return new ListSectionController(providerServices, viewLoader, localeText, watchedService, imageService);
    }

    @Bean
    public LoaderSectionController loaderSectionController() {
        return new LoaderSectionController();
    }

    @Bean
    public MenuSectionController menuSectionController(SettingsService settingsService,
                                                       ApplicationEventPublisher eventPublisher,
                                                       PopcornProperties properties) {
        return new MenuSectionController(settingsService, eventPublisher, properties);
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
    public SettingsSectionController settingsSectionController() {
        return new SettingsSectionController();
    }

    //endregion

    //region Components

    @Bean
    public MovieDetailsComponent movieDetailsComponent(LocaleText localeText,
                                                       ImageService imageService,
                                                       HealthService healthService,
                                                       SettingsService settingsService,
                                                       ApplicationEventPublisher eventPublisher,
                                                       SubtitleService subtitleService,
                                                       FavoriteService favoriteService) {
        return new MovieDetailsComponent(localeText, imageService, healthService, settingsService, eventPublisher, subtitleService, favoriteService);
    }

    @Bean
    public ShowDetailsComponent showDetailsComponent(LocaleText localeText,
                                                     HealthService healthService,
                                                     ImageService imageService,
                                                     SettingsService settingsService,
                                                     ShowHelperService showHelperService,
                                                     WatchedService watchedService,
                                                     ApplicationEventPublisher eventPublisher) {
        return new ShowDetailsComponent(localeText, healthService, imageService, settingsService, showHelperService, watchedService, eventPublisher);
    }

    @Bean
    public SettingsUiComponent settingsUiComponent(ApplicationEventPublisher eventPublisher,
                                                   LocaleText localeText,
                                                   SettingsService settingsService,
                                                   SettingsSectionController settingsSection) {
        return new SettingsUiComponent(eventPublisher, localeText, settingsService, settingsSection);
    }

    @Bean
    public SettingsPlaybackComponent settingsPlaybackComponent(ApplicationEventPublisher eventPublisher,
                                                               LocaleText localeText,
                                                               SettingsService settingsService,
                                                               SettingsSectionController settingsSection) {
        return new SettingsPlaybackComponent(eventPublisher, localeText, settingsService, settingsSection);
    }

    @Bean
    public SettingsSubtitlesComponent settingsSubtitlesComponent(ApplicationEventPublisher eventPublisher,
                                                                 LocaleText localeText,
                                                                 SettingsService settingsService,
                                                                 SettingsSectionController settingsSection) {
        return new SettingsSubtitlesComponent(eventPublisher, localeText, settingsService, settingsSection);
    }

    @Bean
    public PlayerHeaderComponent playerHeaderComponent() {
        return new PlayerHeaderComponent();
    }

    @Bean
    public PlayerControlsComponent playerControlsComponent(VideoPlayerService videoPlayerService,
                                                           VideoPlayerManagerService videoPlayerManagerService,
                                                           VideoPlayerSubtitleService videoPlayerSubtitleService) {
        return new PlayerControlsComponent(videoPlayerService, videoPlayerManagerService, videoPlayerSubtitleService);
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

    //endregion
}
