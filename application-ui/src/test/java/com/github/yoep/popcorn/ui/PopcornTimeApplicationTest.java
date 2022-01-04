package com.github.yoep.popcorn.ui;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.spring.boot.javafx.view.ViewManager;
import com.github.spring.boot.javafx.view.ViewProperties;
import com.github.yoep.popcorn.backend.environment.PlatformProvider;
import com.github.yoep.popcorn.backend.settings.OptionsService;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import com.github.yoep.popcorn.backend.settings.models.ApplicationOptions;
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
    private SettingsService settingsService;
    @Mock
    private OptionsService optionsService;
    @Mock
    private MaximizeService maximizeService;
    @Mock
    private PlatformProvider platformProvider;
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private ViewManager viewManager;

    private PopcornTimeApplication application;

    private final SimpleObjectProperty<Scene> sceneProperty = new SimpleObjectProperty<>();

    @BeforeEach
    void setUp() {
        lenient().when(applicationContext.getBean(SettingsService.class)).thenReturn(settingsService);
        lenient().when(applicationContext.getBean(ViewManager.class)).thenReturn(viewManager);
        lenient().when(applicationContext.getBean(OptionsService.class)).thenReturn(optionsService);
        lenient().when(applicationContext.getBean(PlatformProvider.class)).thenReturn(platformProvider);
        lenient().when(applicationContext.getBean(MaximizeService.class)).thenReturn(maximizeService);
        lenient().when(applicationContext.getBean(ViewLoader.class)).thenReturn(viewLoader);
        lenient().when(stage.sceneProperty()).thenReturn(sceneProperty);

        application = new PopcornTimeApplication(applicationContext);
    }

    @Test
    void testStart_whenNativeUiIsDisabled_shouldRegisterBorderlessStage() throws Exception {
        var options = mock(ApplicationOptions.class);
        var settings = ApplicationSettings.builder()
                .uiSettings(UISettings.builder()
                        .nativeWindowEnabled(false)
                        .build())
                .build();
        when(settingsService.getSettings()).thenReturn(settings);
        when(optionsService.options()).thenReturn(options);
        when(stage.isShowing()).thenReturn(false);

        application.start(stage);

        verify(viewManager).registerPrimaryStage(isA(Stage.class));
        verify(stage, atLeast(1)).initStyle(StageStyle.UNDECORATED);
    }

    @Test
    void testStart_whenNativeUiIsEnabled_shouldNotAddUndecoratedStyle() throws Exception {
        var options = mock(ApplicationOptions.class);
        var settings = ApplicationSettings.builder()
                .uiSettings(UISettings.builder()
                        .nativeWindowEnabled(true)
                        .build())
                .build();
        when(settingsService.getSettings()).thenReturn(settings);
        when(optionsService.options()).thenReturn(options);

        application.start(stage);

        verify(viewManager).registerPrimaryStage(isA(Stage.class));
        verify(stage, times(0)).initStyle(StageStyle.UNDECORATED);
    }

    @Test
    void testStart_whenBigPictureModeIsEnabled_shouldMaximizeOnStartup() throws Exception {
        var options = ApplicationOptions.builder()
                .bigPictureMode(true)
                .build();
        var settings = ApplicationSettings.builder()
                .uiSettings(UISettings.builder()
                        .nativeWindowEnabled(true)
                        .build())
                .build();
        when(settingsService.getSettings()).thenReturn(settings);
        when(optionsService.options()).thenReturn(options);

        application.start(stage);

        verify(maximizeService).setMaximized(true);
    }


    @Test
    void testStart_whenKioskModeIsEnabled_shouldDisableResizing() throws Exception {
        var options = ApplicationOptions.builder()
                .kioskMode(true)
                .build();
        var settings = ApplicationSettings.builder()
                .uiSettings(UISettings.builder()
                        .nativeWindowEnabled(true)
                        .build())
                .build();
        var expectedProperties = ViewProperties.builder()
                .title(PopcornTimeApplication.APPLICATION_TITLE)
                .icon(PopcornTimeApplication.ICON_NAME)
                .resizable(false)
                .centerOnScreen(true)
                .background(Color.BLACK)
                .build();
        when(settingsService.getSettings()).thenReturn(settings);
        when(optionsService.options()).thenReturn(options);
        when(platformProvider.isTransparentWindowSupported()).thenReturn(false);

        application.start(stage);

        verify(viewLoader).show(stage, PopcornTimeApplication.STAGE_VIEW, expectedProperties);
    }
}