package com.github.yoep.popcorn.ui;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.adapters.player.Player;
import com.github.yoep.popcorn.backend.adapters.video.VideoPlayback;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.EventPublisherBridge;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.FxChannelException;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.StartPlayersDiscoveryRequest;
import com.github.yoep.popcorn.backend.loader.LoaderService;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.providers.ProviderServiceImpl;
import com.github.yoep.popcorn.backend.media.tracking.TraktTrackingService;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.backend.player.PlayerManagerServiceImpl;
import com.github.yoep.popcorn.backend.playlists.DefaultPlaylistManager;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.subtitles.SubtitleServiceImpl;
import com.github.yoep.popcorn.backend.torrent.DefaultTorrentService;
import com.github.yoep.popcorn.backend.updater.UpdateService;
import com.github.yoep.popcorn.backend.utils.PopcornLocaleText;
import com.github.yoep.popcorn.backend.utils.ResourceBundleMessageSource;
import com.github.yoep.popcorn.ui.info.PlayerInfoService;
import com.github.yoep.popcorn.ui.info.VideoInfoService;
import com.github.yoep.popcorn.ui.platform.PlatformFX;
import com.github.yoep.popcorn.ui.screen.ScreenServiceImpl;
import com.github.yoep.popcorn.ui.stage.BorderlessStageHolder;
import com.github.yoep.popcorn.ui.stage.BorderlessStageWrapper;
import com.github.yoep.popcorn.ui.torrent.TorrentCollectionService;
import com.github.yoep.popcorn.ui.tracking.EmbeddedAuthorization;
import com.github.yoep.popcorn.ui.utils.PopcornResourceBundleProvider;
import com.github.yoep.popcorn.ui.view.*;
import com.github.yoep.popcorn.ui.view.controllers.ContentSectionController;
import com.github.yoep.popcorn.ui.view.controllers.MainController;
import com.github.yoep.popcorn.ui.view.controllers.common.components.*;
import com.github.yoep.popcorn.ui.view.controllers.common.sections.*;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.*;
import com.github.yoep.popcorn.ui.view.controllers.desktop.sections.TorrentCollectionSectionController;
import com.github.yoep.popcorn.ui.view.controllers.tv.components.*;
import com.github.yoep.popcorn.ui.view.services.*;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.scene.paint.Color;
import javafx.stage.Stage;
import javafx.stage.StageStyle;
import lombok.Getter;
import lombok.NoArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.awt.*;
import java.util.Optional;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
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
        var startTime = System.currentTimeMillis();
        try {
            var fxChannel = IOC.getInstance(FxChannel.class);
            var resourceBundle = IOC.registerInstance(new ResourceBundleMessageSource(new PopcornResourceBundleProvider(), "main", "about", "genres", "languages", "sort-by"));
            var localeText = IOC.registerInstance(new PopcornLocaleText(resourceBundle));
            var applicationConfig = IOC.registerInstance(new ApplicationConfig(fxChannel, localeText));
            var viewManager = IOC.registerInstance(new PopcornViewManager());
            var viewLoader = IOC.registerInstance(new PopcornViewLoader(IOC, applicationConfig, viewManager, localeText));
            var eventPublisher = IOC.registerInstance(new EventPublisher());
            var loaderService = IOC.registerInstance(new LoaderService(fxChannel, eventPublisher));
            var playerManagerService = IOC.registerInstance(new PlayerManagerServiceImpl(fxChannel, eventPublisher));
            var watchedService = IOC.registerInstance(new WatchedService(fxChannel));
            var torrentService = IOC.registerInstance(new DefaultTorrentService(fxChannel));
            var platformProvider = IOC.registerInstance(new PlatformFX());
            var maximizeService = IOC.registerInstance(new MaximizeService(viewManager, applicationConfig));
            var authorization = IOC.registerInstance(new EmbeddedAuthorization(viewLoader, localeText));
            IOC.registerInstance(new SubtitleServiceImpl(fxChannel));
            IOC.registerInstance(new DefaultPlaylistManager(fxChannel, applicationConfig));
            IOC.registerInstance(new EventPublisherBridge(eventPublisher, fxChannel));
            IOC.registerInstance(new ProviderServiceImpl(fxChannel));
            IOC.registerInstance(new FavoriteService(fxChannel));
            IOC.registerInstance(new UrlService(eventPublisher, this, localeText, loaderService));
            IOC.registerInstance(new VideoQualityService(applicationConfig));
            IOC.registerInstance(new ImageService(fxChannel));
            IOC.registerInstance(new ShowHelperService(localeText, watchedService));
            IOC.registerInstance(new SubtitlePickerService(localeText, viewManager));
            IOC.registerInstance(new TorrentCollectionService(fxChannel));

            // services
            IOC.registerInstance(new HealthService(fxChannel, eventPublisher));
            IOC.registerInstance(new PlayerExternalComponentService(playerManagerService, eventPublisher, torrentService));
            IOC.registerInstance(new TorrentSettingService());
            IOC.registerInstance(new TraktTrackingService(fxChannel, authorization));
            IOC.registerInstance(new UpdateService(fxChannel, platformProvider, eventPublisher, localeText));
            IOC.registerInstance(new ScreenServiceImpl(viewManager, applicationConfig, eventPublisher, maximizeService, fxChannel));

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
            IOC.register(ProgressInfoComponent.class);
            IOC.register(SettingsActionsComponent.class);
            IOC.register(ShowDetailsComponent.class);
            IOC.register(TvMediaCardComponent.class);

            // register additional init beans
            Optional.ofNullable(ON_INIT.get())
                    .ifPresent(consumer -> consumer.accept(IOC));

            // register video playback
            var playerInfoService = IOC.registerInstance(new PlayerInfoService(playerManagerService));
            var videoInfoService = IOC.registerInstance(new VideoInfoService(IOC.getInstances(VideoPlayback.class)));
            IOC.registerInstance(new AboutSectionService(playerInfoService, videoInfoService));

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

            if (!applicationConfig.isTvMode()) {
                loadDesktopControllers();
            } else {
                loadTvControllers();
            }

            var players = IOC.getInstances(Player.class);
            log.info("Loaded a total of {} players during the initialization phase", players.size());

            var elapsedTime = System.currentTimeMillis() - startTime;
            log.info("Application initialized in {} seconds", elapsedTime / 1000.0);
        } catch (Exception e) {
            var elapsedTime = System.currentTimeMillis() - startTime;
            log.error("Failed to initialize the application after {} seconds, {}", elapsedTime / 1000.0, e.getMessage(), e);
            throw e;
        }
    }

    @Override
    public void start(Stage stage) throws Exception {
        var startTime = System.currentTimeMillis();
        try {
            log.trace("Starting the application");
            updateStageType(stage);

            log.trace("Loading the main view of the application");
            centerOnActiveScreen(stage);
            var viewManager = IOC.getInstance(ViewManager.class);
            viewManager.setPolicy(ViewManagerPolicy.CLOSEABLE);
            viewManager.registerPrimaryStage(stage);
            var viewProperties = getViewProperties(
                    IOC.getInstance(ApplicationConfig.class),
                    IOC.getInstance(MaximizeService.class),
                    IOC.getInstance(PlatformProvider.class)
            );
            IOC.getInstance(ViewLoader.class).show(stage, STAGE_VIEW, viewProperties);

            log.trace("Starting the discovery of external players");
            var fxChannel = IOC.getInstance(FxChannel.class);
            fxChannel.send(StartPlayersDiscoveryRequest.getDefaultInstance());

            var elapsedTime = System.currentTimeMillis() - startTime;
            log.info("Application started in {} seconds", elapsedTime / 1000.0);
        } catch (Exception ex) {
            log.error("Failed to start the application, {}", ex.getMessage(), ex);
            throw ex;
        }
    }

    @Override
    public void stop() throws Exception {
        super.stop();
        IOC.dispose();
        System.exit(0);
    }

    private void updateStageType(Stage stage) {
        var settingsService = IOC.getInstance(ApplicationConfig.class);
        try {
            var settings = settingsService.getSettings()
                    .thenApply(ApplicationSettings::getUiSettings)
                    .get(10, TimeUnit.SECONDS);

            if (settings.getNativeWindowEnabled()) {
                log.debug("Showing application in window mode");
            } else {
                log.debug("Showing application in borderless window mode");
                BorderlessStageHolder.setWrapper(new BorderlessStageWrapper(stage));
                stage.initStyle(StageStyle.UNDECORATED);
            }
        } catch (ExecutionException | InterruptedException | TimeoutException e) {
            log.error("Failed to update initial stage, {}", e.getMessage(), e);
            throw new FxChannelException(e.getMessage(), e);
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
            applicationConfig.getSettings()
                    .thenApply(ApplicationSettings::getUiSettings)
                    .whenComplete((settings, throwable) -> {
                        if (throwable == null) {
                            maximizeService.setMaximized(settings.getMaximized());
                        } else {
                            log.error("Failed to retrieve settings", throwable);
                        }
                    });
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

    private static void loadDesktopControllers() {
        IOC.register(DesktopFilterComponent.class);
        IOC.register(DesktopMovieActionsComponent.class);
        IOC.register(DesktopMovieQualityComponent.class);
        IOC.register(DesktopSerieActionsComponent.class);
        IOC.register(DesktopSerieQualityComponent.class);
        IOC.register(DesktopSidebarSearchComponent.class);
        IOC.register(DetailsTorrentComponent.class);
        IOC.register(SettingsPlaybackComponent.class);
        IOC.register(SettingsServerComponent.class);
        IOC.register(SettingsSubtitlesComponent.class);
        IOC.register(SettingsTorrentComponent.class);
        IOC.register(SettingsTraktComponent.class);
        IOC.register(SettingsUIComponent.class);
        IOC.register(TorrentCollectionSectionController.class);
        IOC.register(WindowComponent.class);
        IOC.register(PosterComponent.class, false);
    }

    private static void loadTvControllers() {
        IOC.register(TvPosterComponent.class, false);
        IOC.register(TvFilterComponent.class);
        IOC.register(TvMovieActionsComponent.class);
        IOC.register(TvSerieActionsComponent.class);
        IOC.register(TvSerieEpisodeActionsComponent.class);
        IOC.register(TvSettingsServerComponent.class);
        IOC.register(TvSettingsSubtitlesComponent.class);
        IOC.register(TvSettingsTorrentComponent.class);
        IOC.register(TvSettingsUiComponent.class);
        IOC.register(TvSidebarSearchComponent.class);
        IOC.register(SystemTimeComponent.class);
    }
}
