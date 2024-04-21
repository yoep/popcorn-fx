package com.github.yoep.popcorn.ui;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.EventPublisherBridge;
import com.github.yoep.popcorn.backend.loader.LoaderService;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.providers.FavoriteProviderService;
import com.github.yoep.popcorn.backend.media.providers.MovieProviderService;
import com.github.yoep.popcorn.backend.media.providers.ShowProviderService;
import com.github.yoep.popcorn.backend.media.tracking.TraktTrackingService;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.backend.player.PlayerManagerServiceImpl;
import com.github.yoep.popcorn.backend.playlists.PlaylistManager;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.subtitles.SubtitleServiceImpl;
import com.github.yoep.popcorn.backend.updater.UpdateService;
import com.github.yoep.popcorn.backend.utils.PopcornLocaleText;
import com.github.yoep.popcorn.backend.utils.ResourceBundleMessageSource;
import com.github.yoep.popcorn.ui.info.PlayerInfoService;
import com.github.yoep.popcorn.ui.info.VideoInfoService;
import com.github.yoep.popcorn.ui.platform.PlatformFX;
import com.github.yoep.popcorn.ui.stage.BorderlessStageHolder;
import com.github.yoep.popcorn.ui.torrent.TorrentCollectionService;
import com.github.yoep.popcorn.ui.tracking.EmbeddedAuthorization;
import com.github.yoep.popcorn.ui.view.*;
import com.github.yoep.popcorn.ui.view.controllers.ContentSectionController;
import com.github.yoep.popcorn.ui.view.controllers.MainController;
import com.github.yoep.popcorn.ui.view.controllers.common.components.*;
import com.github.yoep.popcorn.ui.view.controllers.common.sections.*;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.*;
import com.github.yoep.popcorn.ui.view.controllers.desktop.sections.TorrentCollectionSectionController;
import com.github.yoep.popcorn.ui.view.controllers.tv.components.TvFilterComponent;
import com.github.yoep.popcorn.ui.view.services.*;
import javafx.application.Application;
import javafx.scene.paint.Color;
import javafx.stage.Stage;
import javafx.stage.StageStyle;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.awt.*;
import java.util.Optional;
import java.util.concurrent.atomic.AtomicReference;
import java.util.function.Consumer;

@Slf4j
@NoArgsConstructor
public class PopcornTimeApplication extends Application {
    public static final String ICON_NAME = "icon_64.png";
    public static final String APPLICATION_TITLE = "Popcorn Time";
    static final String STAGE_VIEW = "main.fxml";

    @Getter
    static final IoC IOC = new IoC();
    @Getter
    static final AtomicReference<Consumer<IoC>> ON_INIT = new AtomicReference<>();

    @Override
    public void init() throws Exception {
        var fxLib = IOC.getInstance(FxLib.class);
        var popcornFx = IOC.getInstance(PopcornFx.class);
        var resourceBundle = IOC.registerInstance(new ResourceBundleMessageSource("main", "about", "genres", "languages", "sort-by"));
        var localeText = IOC.registerInstance(new PopcornLocaleText(resourceBundle));
        var applicationConfig = IOC.registerInstance(new ApplicationConfig(localeText, fxLib, popcornFx));
        var viewManager = IOC.registerInstance(new PopcornViewManager());
        var viewLoader = IOC.registerInstance(new PopcornViewLoader(IOC, applicationConfig, viewManager, localeText));
        var eventPublisher = IOC.registerInstance(new EventPublisher());
        var eventPublisherBridge = IOC.registerInstance(new EventPublisherBridge(eventPublisher, fxLib, popcornFx));
        var maximizeService = IOC.registerInstance(new MaximizeService(viewManager, applicationConfig));
        var platformProvider = IOC.registerInstance(new PlatformFX());
        var loaderService = IOC.registerInstance(new LoaderService(fxLib, popcornFx, eventPublisher));
        var playlistManager = IOC.registerInstance(new PlaylistManager(fxLib, popcornFx, applicationConfig));
        var playerManagerService = new PlayerManagerServiceImpl(fxLib, popcornFx, eventPublisher);
        IOC.registerInstance(new FavoriteProviderService(fxLib, popcornFx));
        IOC.registerInstance(new MovieProviderService(fxLib, popcornFx));
        IOC.registerInstance(new ShowProviderService(fxLib, popcornFx));
        IOC.registerInstance(new FavoriteService(fxLib, popcornFx));
        IOC.registerInstance(new WatchedService(fxLib, popcornFx));
        IOC.registerInstance(playerManagerService);
        IOC.registerInstance(new UrlService(eventPublisher, this, localeText, loaderService));

        // services
        IOC.register(EmbeddedAuthorization.class);
        IOC.register(HealthService.class);
        IOC.register(ImageService.class);
        IOC.register(PlayerExternalComponentService.class);
        IOC.register(ShowHelperService.class);
        IOC.register(SubtitlePickerService.class);
        IOC.register(SubtitleServiceImpl.class);
        IOC.register(TorrentCollectionService.class);
        IOC.register(TorrentSettingService.class);
        IOC.register(TraktTrackingService.class);
        IOC.register(UpdateService.class);
        IOC.register(VideoQualityService.class);

        // components
        IOC.register(EpisodeComponent.class);
        IOC.register(LoaderComponent.class);
        IOC.register(LoadingCardComponent.class);
        IOC.register(MediaCardComponent.class);
        IOC.register(MovieDetailsComponent.class);
        IOC.register(NotificationComponent.class);
        IOC.register(PlayerExternalComponent.class);
        IOC.register(PlayingNextInComponent.class);
        IOC.register(PlaylistItemComponent.class);
        IOC.register(PosterComponent.class);
        IOC.register(ProgressInfoComponent.class);
        IOC.register(SettingsActionsComponent.class);
        IOC.register(ShowDetailsComponent.class);
        IOC.register(TvMediaCardComponent.class);
        IOC.register(TvPosterComponent.class);

        // controllers
        IOC.register(AboutSectionController.class);
        IOC.register(ContentSectionController.class);
        IOC.register(DetailsComponentService.class);
        IOC.register(DetailsSectionController.class);
        IOC.register(LoaderSectionController.class);
        IOC.register(NotificationSectionController.class);
        IOC.register(PlayerSectionController.class);
        IOC.register(SettingsSectionController.class);
        IOC.register(SidebarController.class);
        IOC.register(UpdateSectionController.class);
        IOC.register(ListSectionController.class);
        IOC.register(MainController.class);

        // register additional init beans
        Optional.ofNullable(ON_INIT.get())
                .ifPresent(consumer -> consumer.accept(IOC));

        // register video playback
        var playerInfoService = IOC.registerInstance(new PlayerInfoService(playerManagerService));
        var videoInfoService = IOC.registerInstance(new VideoInfoService(IOC.getInstances(VideoPlayback.class)));
        IOC.registerInstance(new AboutSectionService(playerInfoService, videoInfoService));

        if (!applicationConfig.isTvMode()) {
            loadDesktopControllers(IOC);
        } else {
            loadTvControllers(IOC);
        }
    }

    @Override
    public void start(Stage stage) throws Exception {
        log.trace("Starting the application");
        updateStageType(stage);

        log.trace("Loading the main view of the application");
        centerOnActiveScreen(stage);
        var viewProperties = getViewProperties(
                IOC.getInstance(ApplicationConfig.class),
                IOC.getInstance(MaximizeService.class),
                IOC.getInstance(PlatformProvider.class)
        );
        IOC.getInstance(ViewLoader.class).show(stage, STAGE_VIEW, viewProperties);
        IOC.getInstance(ViewManager.class).setPolicy(ViewManagerPolicy.CLOSEABLE);

        log.trace("Starting the discovery of external players");
        IOC.getInstance(FxLib.class).discover_external_players(IOC.getInstance(PopcornFx.class));
    }

    @Override
    public void stop() throws Exception {
        super.stop();
        IOC.dispose();
        System.exit(0);
    }

    private void updateStageType(Stage stage) {
        var settingsService = IOC.getInstance(ApplicationConfig.class);
        var uiSettings = settingsService.getSettings().getUiSettings();

        if (uiSettings.isNativeWindowEnabled()) {
            log.debug("Showing application in window mode");
        } else {
            log.debug("Showing application in borderless window mode");
            BorderlessStageHolder.setWrapper(new BorderlessStageWrapper(stage));
            stage.initStyle(StageStyle.UNDECORATED);
        }
    }

    private ViewProperties getViewProperties(ApplicationConfig applicationConfig,
                                             MaximizeService maximizeService,
                                             PlatformProvider platformProvider) {
        log.trace("Building the view properties of the application");
        var properties = ViewProperties.builder()
                .title(APPLICATION_TITLE)
                .icon(ICON_NAME)
                .background(getBackgroundColor(platformProvider))
                .centerOnScreen(false);

        // check if the big-picture or kiosk mode or maximized is enabled
        // if so, force the application to be maximized
        if (applicationConfig.isTvMode() || applicationConfig.isMaximized()) {
            maximizeService.setMaximized(true);
        } else {
            var uiSettings = applicationConfig.getSettings().getUiSettings();

            maximizeService.setMaximized(uiSettings.isMaximized());
        }

        // check if the kiosk mode is enabled
        // if so, prevent the application from being resized
        if (applicationConfig.isKioskMode()) {
            properties.resizable(false);
        }

        var viewProperties = properties.build();
        log.debug("Using the following view properties for the application: {}", viewProperties);
        return viewProperties;
    }

    private Color getBackgroundColor(PlatformProvider platformProvider) {
        return platformProvider.isTransparentWindowSupported() ?
                Color.TRANSPARENT : Color.BLACK;
    }

    private static void centerOnActiveScreen(Stage stage) {
        var mouse = MouseInfo.getPointerInfo().getLocation();

        stage.setX(mouse.getX());
        stage.setY(mouse.getY());
        stage.centerOnScreen();
    }

    private static void loadDesktopControllers(IoC instance) {
        instance.register(DesktopFilterComponent.class);
        instance.register(DesktopMovieActionsComponent.class);
        instance.register(DesktopMovieQualityComponent.class);
        instance.register(DesktopSerieActionsComponent.class);
        instance.register(DesktopSerieQualityComponent.class);
        instance.register(DesktopSidebarSearchComponent.class);
        instance.register(DetailsTorrentComponent.class);
        instance.register(SettingsPlaybackComponent.class);
        instance.register(SettingsServerComponent.class);
        instance.register(SettingsSubtitlesComponent.class);
        instance.register(SettingsTorrentComponent.class);
        instance.register(SettingsTraktComponent.class);
        instance.register(SettingsUIComponent.class);
        instance.register(TorrentCollectionSectionController.class);
        instance.register(WindowComponent.class);
    }

    private static void loadTvControllers(IoC instance) {
        instance.register(TvFilterComponent.class);
    }
}
