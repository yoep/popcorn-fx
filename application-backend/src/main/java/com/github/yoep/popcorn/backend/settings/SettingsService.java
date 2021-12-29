package com.github.yoep.popcorn.backend.settings;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.UIScale;
import com.github.yoep.popcorn.backend.settings.models.UISettings;
import com.github.yoep.popcorn.backend.storage.StorageService;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;
import java.util.List;
import java.util.Locale;
import java.util.Optional;

import static java.util.Arrays.asList;

@Slf4j
@Service
@RequiredArgsConstructor
public class SettingsService {
    private static final String STORAGE_NAME = "settings.json";
    private final StorageService storageService;
    private final ViewLoader viewLoader;
    private final OptionsService optionsService;
    private final LocaleText localeText;

    private ApplicationSettings currentSettings;

    //region Getters

    /**
     * Get the application settings.
     *
     * @return Returns the application settings.
     */
    public ApplicationSettings getSettings() {
        return currentSettings;
    }

    //endregion

    //region Methods

    /**
     * Increases the current UI scale.
     */
    public void increaseUIScale() {
        var index = getCurrentUIScaleIndex();
        var supportedUIScales = supportedUIScales();

        // verify that the current UI scale is not the max supported scale
        if (index == supportedUIScales.size() - 1)
            return;

        var uiSettings = getSettings().getUiSettings();
        uiSettings.setUiScale(supportedUIScales.get(index + 1));
    }

    /**
     * Decrease the current UI scale.
     */
    public void decreaseUIScale() {
        var index = getCurrentUIScaleIndex();
        var supportedUIScales = supportedUIScales();

        // verify that the current UI scale is the min supported scale
        if (index == 0)
            return;

        var uiSettings = getSettings().getUiSettings();
        uiSettings.setUiScale(supportedUIScales.get(index - 1));
    }

    /**
     * Save the current application settings to the {@link #STORAGE_NAME} file.
     */
    public void save() {
        save(currentSettings);
    }

    /**
     * Save the given application settings.
     * This replaces the currently stored settings.
     *
     * @param settings The application settings to save.
     */
    public void save(ApplicationSettings settings) {
        if (settings != currentSettings)
            currentSettings = settings;

        log.debug("Saving application settings to storage");
        storageService.store(STORAGE_NAME, settings);
        log.info("Application settings have been saved");
    }

    /**
     * Get the list of supported UI scales for this application.
     *
     * @return Returns a list of supported UI scales.
     */
    public static List<UIScale> supportedUIScales() {
        return asList(
                new UIScale(0.25f),
                new UIScale(0.5f),
                new UIScale(0.75f),
                UISettings.DEFAULT_UI_SCALE,
                new UIScale(1.25f),
                new UIScale(1.50f),
                new UIScale(2.0f),
                new UIScale(3.0f),
                new UIScale(4.0f),
                new UIScale(5.0f)
        );
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        initializeSettings();
        initializeDefaultLanguage();
    }

    private void initializeSettings() {
        this.currentSettings = loadSettingsFromFile().orElse(ApplicationSettings.builder().build());
        var uiSettings = this.currentSettings.getUiSettings();

        uiSettings.addListener(event -> {
            if (event.getPropertyName().equals(UISettings.UI_SCALE_PROPERTY)) {
                var uiScale = (UIScale) event.getNewValue();

                updateUIScale(uiScale.getValue());
            }
        });

        updateUIScale(uiSettings.getUiScale().getValue());
    }

    private void initializeDefaultLanguage() {
        var uiSettings = this.currentSettings.getUiSettings();

        // update the locale text with the locale from the settings
        localeText.updateLocale(uiSettings.getDefaultLanguage());

        // add a listener to the default language for changing the language at runtime
        uiSettings.addListener(event -> {
            if (event.getPropertyName().equals(UISettings.LANGUAGE_PROPERTY)) {
                var locale = (Locale) event.getNewValue();

                localeText.updateLocale(locale);
            }
        });
    }

    //endregion

    //region PreDestroy

    @PreDestroy
    private void destroy() {
        save();
    }

    //endregion

    //region Functions

    private void updateUIScale(float scale) {
        var options = optionsService.options();

        // check if the big-picture mode is activated
        // if so, double the UI scale
        if (options.isBigPictureMode())
            scale *= 2;

        viewLoader.setScale(scale);
    }

    private Optional<ApplicationSettings> loadSettingsFromFile() {
        log.debug("Loading application settings from storage");
        return storageService.read(STORAGE_NAME, ApplicationSettings.class);
    }

    private int getCurrentUIScaleIndex() {
        var uiSettings = currentSettings.getUiSettings();
        var scale = uiSettings.getUiScale();
        var index = supportedUIScales().indexOf(scale);

        // check if the index was found
        // if not, return the index of the default
        if (index == -1) {
            log.warn("UI scale \"{}\" couldn't be found back in the supported UI scales", scale);
            index = supportedUIScales().indexOf(UISettings.DEFAULT_UI_SCALE);
        }

        log.trace("Current UI scale index: {}", index);
        return index;
    }

    //endregion
}
