package com.github.yoep.popcorn.ui;

import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.*;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import com.github.yoep.popcorn.ui.view.ViewManager;
import com.github.yoep.popcorn.ui.view.ViewProperties;
import com.github.yoep.popcorn.ui.view.controllers.MainController;
import com.github.yoep.popcorn.ui.view.services.MaximizeService;
import javafx.application.Platform;
import javafx.beans.property.SimpleObjectProperty;
import javafx.scene.Scene;
import javafx.scene.paint.Color;
import javafx.stage.Stage;
import javafx.stage.StageStyle;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.util.concurrent.ExecutorService;

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class PopcornTimeApplicationTest {
    @Mock
    private Stage stage;
    @Mock
    private MaximizeService maximizeService;
    @Mock
    private PlatformProvider platformProvider;
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private ViewManager viewManager;
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private ExecutorService executorService;
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx popcornFx;

    private final SimpleObjectProperty<Scene> sceneProperty = new SimpleObjectProperty<>();

    @BeforeEach
    void setUp() {
        lenient().when(stage.sceneProperty()).thenReturn(sceneProperty);

        PopcornTimeApplication.IOC.registerInstance(fxLib);
        PopcornTimeApplication.IOC.registerInstance(popcornFx);
        PopcornTimeApplication.IOC.registerInstance(new ApplicationArgs(new String[]{}));
    }

    @AfterEach
    void tearDown() {
        PopcornTimeApplication.IOC.dispose();
    }

    @Test
    void testInit() throws Exception {
        var settings = mock(ApplicationSettings.class);
        var uiSettings = mock(UISettings.class);
        when(fxLib.application_settings(popcornFx)).thenReturn(settings);
        when(settings.getUiSettings()).thenReturn(uiSettings);
        when(uiSettings.getUiScale()).thenReturn(mock(UIScale.class));
        when(uiSettings.getDefaultLanguage()).thenReturn("en/US");
        var application = new PopcornTimeApplication();
        application.init();

        var result = PopcornTimeApplication.IOC.getInstance(MainController.class);

        assertNotNull(result);
    }

    @Test
    void testStart_whenNativeUiIsDisabled_shouldRegisterBorderlessStage() {
        var settings = ApplicationSettings.builder()
                .uiSettings(UISettings.builder()
                        .defaultLanguage("en/US")
                        .nativeWindowEnabled((byte) 0)
                        .uiScale(mock(UIScale.class))
                        .build())
                .subtitleSettings(mock(SubtitleSettings.class))
                .torrentSettings(mock(TorrentSettings.class))
                .serverSettings(mock(ServerSettings.class))
                .playbackSettings(mock(PlaybackSettings.class))
                .trackingSettings(mock(TrackingSettings.class))
                .build();
        PopcornTimeApplication.IOC.registerInstance(applicationConfig);
        PopcornTimeApplication.IOC.registerInstance(executorService);
        PopcornTimeApplication.IOC.registerInstance(maximizeService);
        PopcornTimeApplication.IOC.registerInstance(platformProvider);
        PopcornTimeApplication.IOC.registerInstance(viewManager);
        PopcornTimeApplication.IOC.registerInstance(viewLoader);
        when(applicationConfig.getSettings()).thenReturn(settings);
        when(stage.isShowing()).thenReturn(false);
        var application = new PopcornTimeApplication();

        Platform.runLater(() -> {
            try {
                application.start(stage);
            } catch (Exception e) {
                throw new RuntimeException(e);
            }
        });
        WaitForAsyncUtils.waitForFxEvents();

        verify(viewManager).registerPrimaryStage(isA(Stage.class));
        verify(stage, atLeast(1)).initStyle(StageStyle.UNDECORATED);
    }

    @Test
    void testStart_whenNativeUiIsEnabled_shouldNotAddUndecoratedStyle() throws Exception {
        var settings = ApplicationSettings.builder()
                .uiSettings(UISettings.builder()
                        .defaultLanguage("en/US")
                        .nativeWindowEnabled((byte) 1)
                        .uiScale(mock(UIScale.class))
                        .build())
                .subtitleSettings(mock(SubtitleSettings.class))
                .torrentSettings(mock(TorrentSettings.class))
                .serverSettings(mock(ServerSettings.class))
                .playbackSettings(mock(PlaybackSettings.class))
                .trackingSettings(mock(TrackingSettings.class))
                .build();
        PopcornTimeApplication.IOC.registerInstance(applicationConfig);
        PopcornTimeApplication.IOC.registerInstance(executorService);
        PopcornTimeApplication.IOC.registerInstance(maximizeService);
        PopcornTimeApplication.IOC.registerInstance(platformProvider);
        PopcornTimeApplication.IOC.registerInstance(viewManager);
        PopcornTimeApplication.IOC.registerInstance(viewLoader);
        when(applicationConfig.getSettings()).thenReturn(settings);
        var application = new PopcornTimeApplication();

        Platform.runLater(() -> {
            try {
                application.start(stage);
            } catch (Exception e) {
                throw new RuntimeException(e);
            }
        });
        WaitForAsyncUtils.waitForFxEvents();

        verify(viewManager).registerPrimaryStage(isA(Stage.class));
        verify(stage, times(0)).initStyle(StageStyle.UNDECORATED);
    }

    @Test
    void testStart_whenTvModeIsEnabled_shouldMaximizeOnStartup() throws Exception {
        var settings = ApplicationSettings.builder()
                .uiSettings(UISettings.builder()
                        .defaultLanguage("en/US")
                        .nativeWindowEnabled((byte) 1)
                        .uiScale(mock(UIScale.class))
                        .build())
                .subtitleSettings(mock(SubtitleSettings.class))
                .torrentSettings(mock(TorrentSettings.class))
                .serverSettings(mock(ServerSettings.class))
                .playbackSettings(mock(PlaybackSettings.class))
                .trackingSettings(mock(TrackingSettings.class))
                .build();
        PopcornTimeApplication.IOC.registerInstance(applicationConfig);
        PopcornTimeApplication.IOC.registerInstance(executorService);
        PopcornTimeApplication.IOC.registerInstance(maximizeService);
        PopcornTimeApplication.IOC.registerInstance(platformProvider);
        PopcornTimeApplication.IOC.registerInstance(viewManager);
        PopcornTimeApplication.IOC.registerInstance(viewLoader);
        when(applicationConfig.getSettings()).thenReturn(settings);
        when(applicationConfig.isTvMode()).thenReturn(true);
        var application = new PopcornTimeApplication();

        Platform.runLater(() -> {
            try {
                application.start(stage);
            } catch (Exception e) {
                throw new RuntimeException(e);
            }
        });
        WaitForAsyncUtils.waitForFxEvents();

        verify(maximizeService).setMaximized(true);
    }


    @Test
    void testStart_whenKioskModeIsEnabled_shouldDisableResizing() throws Exception {
        var settings = ApplicationSettings.builder()
                .uiSettings(UISettings.builder()
                        .defaultLanguage("en/US")
                        .nativeWindowEnabled((byte) 1)
                        .uiScale(mock(UIScale.class))
                        .build())
                .subtitleSettings(mock(SubtitleSettings.class))
                .torrentSettings(mock(TorrentSettings.class))
                .serverSettings(mock(ServerSettings.class))
                .playbackSettings(mock(PlaybackSettings.class))
                .trackingSettings(mock(TrackingSettings.class))
                .build();
        var expectedProperties = ViewProperties.builder()
                .title(PopcornTimeApplication.APPLICATION_TITLE)
                .icon(PopcornTimeApplication.ICON_NAME)
                .resizable(false)
                .centerOnScreen(false)
                .background(Color.BLACK)
                .build();
        PopcornTimeApplication.IOC.registerInstance(applicationConfig);
        PopcornTimeApplication.IOC.registerInstance(executorService);
        PopcornTimeApplication.IOC.registerInstance(maximizeService);
        PopcornTimeApplication.IOC.registerInstance(platformProvider);
        PopcornTimeApplication.IOC.registerInstance(viewManager);
        PopcornTimeApplication.IOC.registerInstance(viewLoader);
        when(applicationConfig.getSettings()).thenReturn(settings);
        when(applicationConfig.isKioskMode()).thenReturn(true);
        when(platformProvider.isTransparentWindowSupported()).thenReturn(false);
        var application = new PopcornTimeApplication();

        Platform.runLater(() -> {
            try {
                application.start(stage);
            } catch (Exception e) {
                throw new RuntimeException(e);
            }
        });
        WaitForAsyncUtils.waitForFxEvents();

        verify(viewLoader).show(stage, PopcornTimeApplication.STAGE_VIEW, expectedProperties);
    }

    @Test
    void testStart_shouldStartExternalPlayerDiscovery() {
        var settings = ApplicationSettings.builder()
                .uiSettings(UISettings.builder()
                        .nativeWindowEnabled((byte) 1)
                        .build())
                .build();
        PopcornTimeApplication.IOC.registerInstance(applicationConfig);
        PopcornTimeApplication.IOC.registerInstance(executorService);
        PopcornTimeApplication.IOC.registerInstance(maximizeService);
        PopcornTimeApplication.IOC.registerInstance(platformProvider);
        PopcornTimeApplication.IOC.registerInstance(viewManager);
        PopcornTimeApplication.IOC.registerInstance(viewLoader);
        when(applicationConfig.getSettings()).thenReturn(settings);
        var application = new PopcornTimeApplication();

        Platform.runLater(() -> {
            try {
                application.start(stage);
            } catch (Exception e) {
                throw new RuntimeException(e);
            }
        });
        WaitForAsyncUtils.waitForFxEvents();

        verify(fxLib).discover_external_players(popcornFx);
    }
}