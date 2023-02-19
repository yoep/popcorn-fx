package com.github.yoep.popcorn.backend.settings;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.settings.models.ApplicationOptions;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.UIScale;
import com.github.yoep.popcorn.backend.settings.models.UISettings;
import com.github.yoep.popcorn.backend.storage.StorageService;
import org.junit.jupiter.api.Nested;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.Locale;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class SettingsServiceTest {
    @Mock
    private StorageService storageService;
    @Mock
    private ViewLoader viewLoader;
    @Mock
    private OptionsService optionsService;
    @Mock
    private LocaleText localeText;
    @InjectMocks
    private SettingsService settingsService;

    @Nested
    class UiScaleTest {
        @Test
        void testIncreaseUIScale_whenUiScaleIsNotAtMaximum_shouldIncreaseUiScale() {
            var currentScale = new UIScale(1.50f);
            var settings = ApplicationSettings.builder()
                    .uiSettings(UISettings.builder()
                            .uiScale(currentScale)
                            .build())
                    .build();
            var expectedScale = SettingsService.supportedUIScales().get(SettingsService.supportedUIScales().indexOf(currentScale) + 1);
            when(storageService.read(SettingsService.STORAGE_NAME, ApplicationSettings.class)).thenReturn(Optional.of(settings));
            when(optionsService.options()).thenReturn(ApplicationOptions.builder().build());

            settingsService.init();
            settingsService.increaseUIScale();

            var result = settingsService.getSettings().getUiSettings().getUiScale();
            assertEquals(expectedScale, result);
        }

        @Test
        void testIncreaseUIScale_whenUiScaleIsAtMaximum_shouldNotIncreaseUiScale() {
            var currentScale = SettingsService.supportedUIScales().get(SettingsService.supportedUIScales().size() - 1);
            var settings = ApplicationSettings.builder()
                    .uiSettings(UISettings.builder()
                            .uiScale(currentScale)
                            .build())
                    .build();
            when(storageService.read(SettingsService.STORAGE_NAME, ApplicationSettings.class)).thenReturn(Optional.of(settings));
            when(optionsService.options()).thenReturn(ApplicationOptions.builder().build());

            settingsService.init();
            settingsService.increaseUIScale();

            var result = settingsService.getSettings().getUiSettings().getUiScale();
            assertEquals(currentScale, result, "Expected the ui scale to not have been increased");
        }

        @Test
        void testDecreaseUIScale_whenUiScaleIsNotAtMinimum_shouldDecreaseUiScale() {
            var currentScale = new UIScale(1.50f);
            var settings = ApplicationSettings.builder()
                    .uiSettings(UISettings.builder()
                            .uiScale(currentScale)
                            .build())
                    .build();
            var expectedScale = SettingsService.supportedUIScales().get(SettingsService.supportedUIScales().indexOf(currentScale) - 1);
            when(storageService.read(SettingsService.STORAGE_NAME, ApplicationSettings.class)).thenReturn(Optional.of(settings));
            when(optionsService.options()).thenReturn(ApplicationOptions.builder().build());

            settingsService.init();
            settingsService.decreaseUIScale();

            var result = settingsService.getSettings().getUiSettings().getUiScale();
            assertEquals(expectedScale, result);
        }

        @Test
        void testDecreaseUIScale_whenUiScaleIsAtMinimum_shouldNotDecreaseUiScale() {
            var currentScale = SettingsService.supportedUIScales().get(0);
            var settings = ApplicationSettings.builder()
                    .uiSettings(UISettings.builder()
                            .uiScale(currentScale)
                            .build())
                    .build();
            when(storageService.read(SettingsService.STORAGE_NAME, ApplicationSettings.class)).thenReturn(Optional.of(settings));
            when(optionsService.options()).thenReturn(ApplicationOptions.builder().build());

            settingsService.init();
            settingsService.decreaseUIScale();

            var result = settingsService.getSettings().getUiSettings().getUiScale();
            assertEquals(currentScale, result, "Expected the ui scale to not have been decreased");
        }
    }

    @Nested
    class SaveTest {
        @Test
        void testSave_whenInvoked_shouldStoreTheCurrentSettingsInTheStorage() {
            var settings = ApplicationSettings.builder().build();
            var updatedLanguage = Locale.ENGLISH;
            var expectedSettings = ApplicationSettings.builder()
                    .uiSettings(UISettings.builder()
                            .defaultLanguage(updatedLanguage)
                            .build())
                    .build();
            when(storageService.read(SettingsService.STORAGE_NAME, ApplicationSettings.class)).thenReturn(Optional.of(settings));
            when(optionsService.options()).thenReturn(ApplicationOptions.builder().build());

            settingsService.init();
            settingsService.getSettings().getUiSettings().setDefaultLanguage(updatedLanguage);
            settingsService.save();

            verify(storageService).store(SettingsService.STORAGE_NAME, expectedSettings);
        }

        @Test
        void testSave_whenSettingsArePassed_shouldStoreTheGivenSettingsInTheStorage() {
            var settings = ApplicationSettings.builder().build();
            var newSettings = ApplicationSettings.builder()
                    .uiSettings(UISettings.builder()
                            .uiScale(new UIScale(2f))
                            .build())
                    .build();
            when(storageService.read(SettingsService.STORAGE_NAME, ApplicationSettings.class)).thenReturn(Optional.of(settings));
            when(optionsService.options()).thenReturn(ApplicationOptions.builder().build());

            settingsService.init();
            settingsService.save(newSettings);

            verify(storageService).store(SettingsService.STORAGE_NAME, newSettings);
        }
    }

    @Nested
    class InitTest{
        @Test
        void testInit_whenStorageIsEmpty_shouldCreateNewSettings() {
            var expectedResult = ApplicationSettings.builder().build();
            when(storageService.read(SettingsService.STORAGE_NAME, ApplicationSettings.class)).thenReturn(Optional.empty());
            when(optionsService.options()).thenReturn(ApplicationOptions.builder().build());

            settingsService.init();
            var result = settingsService.getSettings();

            assertEquals(expectedResult, result);
        }

        @Test
        void testInit_whenBigPictureIsEnabled_shouldDoubleTheScale() {
            var initialScale = 1.5f;
            var settings = ApplicationSettings.builder()
                    .uiSettings(UISettings.builder()
                            .uiScale(new UIScale(initialScale))
                            .build())
                    .build();
            when(storageService.read(SettingsService.STORAGE_NAME, ApplicationSettings.class)).thenReturn(Optional.of(settings));
            when(optionsService.options()).thenReturn(ApplicationOptions.builder()
                            .bigPictureMode(true)
                    .build());

            settingsService.init();

            verify(viewLoader).setScale(3f);
        }
    }
}