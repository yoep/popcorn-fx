package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationSettings;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.controls.AxisItemSelection;
import com.github.yoep.popcorn.ui.view.controls.Overlay;
import javafx.scene.control.Button;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.when;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class TvSettingsUiComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private LocaleText localeText;
    @Mock
    private ApplicationConfig settingsService;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    private TvSettingsUiComponent component;

    @BeforeEach
    void setUp() {
        component = new TvSettingsUiComponent(eventPublisher, localeText, settingsService);

        component.defaultLanguage = new Button();
        component.languages = new AxisItemSelection<>();
        component.defaultLanguageOverlay = new Overlay();

        component.uiScale = new Button();
        component.uiScales = new AxisItemSelection<>();

        component.startScreen = new Button();
        component.startScreens = new AxisItemSelection<>();
    }

    @Test
    void testInitialize_shouldLoadSettings() throws TimeoutException {
        var language = "English";
        var languageText = "MyLanguage";
        var scaleText = "120%";
        when(settingsService.getSettings()).thenReturn(CompletableFuture.completedFuture(ApplicationSettings.newBuilder()
                .setUiSettings(ApplicationSettings.UISettings.newBuilder()
                        .setDefaultLanguage(language)
                        .setStartScreen(Media.Category.FAVORITES)
                        .setScale(ApplicationSettings.UISettings.Scale.newBuilder()
                                .setFactor(1.2f)
                                .build())
                        .build())
                .build()));
        when(localeText.get(isA(String.class))).thenReturn("FooBar");
        when(localeText.get("language_en")).thenReturn(languageText);
        when(localeText.get("language_english")).thenReturn(languageText);
        component.initialize(url, resourceBundle);

        WaitForAsyncUtils.waitForFxEvents();
        WaitForAsyncUtils.waitFor(500, TimeUnit.MILLISECONDS, () -> component.defaultLanguage.getText() != null);

        assertEquals(languageText, component.defaultLanguage.getText());
        assertEquals(scaleText, component.uiScale.getText());
    }
}