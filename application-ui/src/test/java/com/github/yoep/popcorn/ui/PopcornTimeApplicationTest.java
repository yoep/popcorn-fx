package com.github.yoep.popcorn.ui;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.spring.boot.javafx.view.ViewManager;
import com.github.spring.boot.javafx.view.ViewProperties;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.adapters.platform.PlatformProvider;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.UISettings;
import com.github.yoep.popcorn.ui.view.services.MaximizeService;
import javafx.beans.property.SimpleObjectProperty;
import javafx.scene.Scene;
import javafx.scene.paint.Color;
import javafx.stage.Stage;
import javafx.stage.StageStyle;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.ConfigurableApplicationContext;

import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class PopcornTimeApplicationTest {
    @Mock
    private Stage stage;
    @Mock
    private ConfigurableApplicationContext applicationContext;
    @Mock
    private ApplicationConfig applicationConfig;
    @Mock
    private MaximizeService maximizeService;
    @Mock
    private PlatformProvider platformProvider;
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private ViewManager viewManager;
    @Mock
    private FxLib fxLib;
    @Mock
    private PopcornFx popcornFx;

    private PopcornTimeApplication application;

    private final SimpleObjectProperty<Scene> sceneProperty = new SimpleObjectProperty<>();

    @BeforeEach
    void setUp() {
        lenient().when(applicationContext.getBean(ApplicationConfig.class)).thenReturn(applicationConfig);
        lenient().when(applicationContext.getBean(ViewManager.class)).thenReturn(viewManager);
        lenient().when(applicationContext.getBean(PlatformProvider.class)).thenReturn(platformProvider);
        lenient().when(applicationContext.getBean(MaximizeService.class)).thenReturn(maximizeService);
        lenient().when(applicationContext.getBean(ViewLoader.class)).thenReturn(viewLoader);
        lenient().when(applicationContext.getBean(FxLib.class)).thenReturn(fxLib);
        lenient().when(applicationContext.getBean(PopcornFx.class)).thenReturn(popcornFx);
        lenient().when(stage.sceneProperty()).thenReturn(sceneProperty);

        application = new PopcornTimeApplication(applicationContext);
    }

    @Test
    void testStart_whenNativeUiIsDisabled_shouldRegisterBorderlessStage() throws Exception {
        var settings = ApplicationSettings.builder()
                .uiSettings(UISettings.builder()
                        .nativeWindowEnabled((byte) 0)
                        .build())
                .build();
        when(applicationConfig.getSettings()).thenReturn(settings);
        when(stage.isShowing()).thenReturn(false);

        application.start(stage);

        verify(viewManager).registerPrimaryStage(isA(Stage.class));
        verify(stage, atLeast(1)).initStyle(StageStyle.UNDECORATED);
    }

    @Test
    void testStart_whenNativeUiIsEnabled_shouldNotAddUndecoratedStyle() throws Exception {
        var settings = ApplicationSettings.builder()
                .uiSettings(UISettings.builder()
                        .nativeWindowEnabled((byte) 1)
                        .build())
                .build();
        when(applicationConfig.getSettings()).thenReturn(settings);

        application.start(stage);

        verify(viewManager).registerPrimaryStage(isA(Stage.class));
        verify(stage, times(0)).initStyle(StageStyle.UNDECORATED);
    }

    @Test
    void testStart_whenTvModeIsEnabled_shouldMaximizeOnStartup() throws Exception {
        var settings = ApplicationSettings.builder()
                .uiSettings(UISettings.builder()
                        .nativeWindowEnabled((byte) 1)
                        .build())
                .build();
        when(applicationConfig.getSettings()).thenReturn(settings);
        when(applicationConfig.isTvMode()).thenReturn(true);

        application.start(stage);

        verify(maximizeService).setMaximized(true);
    }


    @Test
    void testStart_whenKioskModeIsEnabled_shouldDisableResizing() throws Exception {
        var settings = ApplicationSettings.builder()
                .uiSettings(UISettings.builder()
                        .nativeWindowEnabled((byte) 1)
                        .build())
                .build();
        var expectedProperties = ViewProperties.builder()
                .title(PopcornTimeApplication.APPLICATION_TITLE)
                .icon(PopcornTimeApplication.ICON_NAME)
                .resizable(false)
                .centerOnScreen(false)
                .background(Color.BLACK)
                .build();
        when(applicationConfig.getSettings()).thenReturn(settings);
        when(applicationConfig.isKioskMode()).thenReturn(true);
        when(platformProvider.isTransparentWindowSupported()).thenReturn(false);

        application.start(stage);

        verify(viewLoader).show(stage, PopcornTimeApplication.STAGE_VIEW, expectedProperties);
    }

    @Test
    void testStart_shouldStartExternalPlayerDiscovery() throws Exception {
        var settings = ApplicationSettings.builder()
                .uiSettings(UISettings.builder()
                        .nativeWindowEnabled((byte) 1)
                        .build())
                .build();
        when(applicationConfig.getSettings()).thenReturn(settings);

        application.start(stage);

        verify(fxLib).discover_external_players(popcornFx);
    }
}