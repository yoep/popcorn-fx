package com.github.yoep.popcorn.ui;

import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.StartPlayersDiscoveryRequest;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.view.ViewLoader;
import com.github.yoep.popcorn.ui.view.ViewManager;
import com.github.yoep.popcorn.ui.view.ViewProperties;
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

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

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
    private FxChannel fxChannel;

    private final SimpleObjectProperty<Scene> sceneProperty = new SimpleObjectProperty<>();

    @BeforeEach
    void setUp() {
        lenient().when(stage.sceneProperty()).thenReturn(sceneProperty);

        PopcornTimeApplication.IOC.registerInstance(fxChannel);
        PopcornTimeApplication.IOC.registerInstance(Executors.newCachedThreadPool(e -> new Thread(e, "popcorn-fx")));
        PopcornTimeApplication.IOC.registerInstance(new ApplicationArgs(new String[]{}));
    }

    @AfterEach
    void tearDown() {
        PopcornTimeApplication.IOC.dispose();
    }

    @Test
    void testStart_whenNativeUiIsDisabled_shouldRegisterBorderlessStage() {
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setUiSettings(ApplicationSettings.UISettings.newBuilder()
                        .setDefaultLanguage("en/US")
                        .setNativeWindowEnabled(false)
                        .setScale(ApplicationSettings.UISettings.Scale.newBuilder()
                                .setFactor(1.0f)
                                .build())
                        .build())
                .build()));
        PopcornTimeApplication.IOC.registerInstance(applicationConfig);
        PopcornTimeApplication.IOC.registerInstance(executorService);
        PopcornTimeApplication.IOC.registerInstance(maximizeService);
        PopcornTimeApplication.IOC.registerInstance(platformProvider);
        PopcornTimeApplication.IOC.registerInstance(viewManager);
        PopcornTimeApplication.IOC.registerInstance(viewLoader);
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
    void testStart_whenNativeUiIsEnabled_shouldNotAddUndecoratedStyle() {
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setUiSettings(ApplicationSettings.UISettings.newBuilder()
                        .setDefaultLanguage("en/US")
                        .setNativeWindowEnabled(true)
                        .setScale(ApplicationSettings.UISettings.Scale.newBuilder()
                                .setFactor(1.0f)
                                .build())
                        .build())
                .build()));
        PopcornTimeApplication.IOC.registerInstance(applicationConfig);
        PopcornTimeApplication.IOC.registerInstance(executorService);
        PopcornTimeApplication.IOC.registerInstance(maximizeService);
        PopcornTimeApplication.IOC.registerInstance(platformProvider);
        PopcornTimeApplication.IOC.registerInstance(viewManager);
        PopcornTimeApplication.IOC.registerInstance(viewLoader);
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
    void testStart_whenTvModeIsEnabled_shouldMaximizeOnStartup() {
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setUiSettings(ApplicationSettings.UISettings.newBuilder()
                        .setDefaultLanguage("en/US")
                        .setScale(ApplicationSettings.UISettings.Scale.newBuilder()
                                .setFactor(1.0f)
                                .build())
                        .build())
                .build()));
        PopcornTimeApplication.IOC.registerInstance(applicationConfig);
        PopcornTimeApplication.IOC.registerInstance(executorService);
        PopcornTimeApplication.IOC.registerInstance(maximizeService);
        PopcornTimeApplication.IOC.registerInstance(platformProvider);
        PopcornTimeApplication.IOC.registerInstance(viewManager);
        PopcornTimeApplication.IOC.registerInstance(viewLoader);
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
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setUiSettings(ApplicationSettings.UISettings.newBuilder()
                        .setDefaultLanguage("en/US")
                        .setScale(ApplicationSettings.UISettings.Scale.newBuilder()
                                .setFactor(1.0f)
                                .build())
                        .build())
                .build()));
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
        when(applicationConfig.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setUiSettings(ApplicationSettings.UISettings.newBuilder()
                        .setDefaultLanguage("en/US")
                        .setNativeWindowEnabled(true)
                        .setScale(ApplicationSettings.UISettings.Scale.newBuilder()
                                .setFactor(1.0f)
                                .build())
                        .build())
                .build()));
        PopcornTimeApplication.IOC.registerInstance(applicationConfig);
        PopcornTimeApplication.IOC.registerInstance(executorService);
        PopcornTimeApplication.IOC.registerInstance(maximizeService);
        PopcornTimeApplication.IOC.registerInstance(platformProvider);
        PopcornTimeApplication.IOC.registerInstance(viewManager);
        PopcornTimeApplication.IOC.registerInstance(viewLoader);
        var application = new PopcornTimeApplication();

        Platform.runLater(() -> {
            try {
                application.start(stage);
            } catch (Exception e) {
                throw new RuntimeException(e);
            }
        });
        WaitForAsyncUtils.waitForFxEvents();

        verify(fxChannel).send(isA(StartPlayersDiscoveryRequest.class));
    }
}