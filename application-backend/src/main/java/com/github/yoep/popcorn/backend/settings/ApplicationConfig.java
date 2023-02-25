package com.github.yoep.popcorn.backend.settings;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import com.github.yoep.popcorn.backend.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.backend.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.backend.settings.models.TorrentSettings;
import com.github.yoep.popcorn.backend.settings.models.UIScale;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.util.List;
import java.util.Objects;
import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedDeque;

import static java.util.Arrays.asList;

@Slf4j
@Service
@RequiredArgsConstructor
public class ApplicationConfig {
    private final ViewLoader viewLoader;
    private final OptionsService optionsService;
    private final LocaleText localeText;
    private final Queue<ApplicationConfigEventCallback> listeners = new ConcurrentLinkedDeque<>();
    private final ApplicationConfigEventCallback callback = createCallback();

    private ApplicationProperties properties;
    private ApplicationSettings settings;

    //region Getters

    public ApplicationProperties getProperties() {
        return properties;
    }

    /**
     * Get the application settings.
     *
     * @return Returns the application settings.
     */
    public ApplicationSettings getSettings() {
        return settings;
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
        save(settings);
    }

    /**
     * Save the given application settings.
     * This replaces the currently stored settings.
     *
     * @param settings The application settings to save.
     */
    public void save(ApplicationSettings settings) {
        if (settings != this.settings)
            this.settings = settings;

        log.debug("Saving application settings to storage");
        //        storageService.store(STORAGE_NAME, settings);
        log.info("Application settings have been saved");
    }

    public void register(ApplicationConfigEventCallback callback) {
        Objects.requireNonNull(callback, "callback cannot be null");
        listeners.add(callback);
    }

    /**
     * Update the subtitle settings of the application with the new value.
     *
     * @param settings The new settings to use.
     */
    public void update(SubtitleSettings settings) {
        Objects.requireNonNull(settings, "settings cannot be null");
        var settings_c = new SubtitleSettings.ByValue(settings);
        FxLib.INSTANCE.update_subtitle_settings(PopcornFxInstance.INSTANCE.get(), settings_c);
    }

    /**
     * Update the subtitle settings of the application with the new value.
     *
     * @param settings The new settings to use.
     */
    public void update(TorrentSettings settings) {
        Objects.requireNonNull(settings, "settings cannot be null");
        var settings_c = new TorrentSettings.ByValue(settings);
        FxLib.INSTANCE.update_torrent_settings(PopcornFxInstance.INSTANCE.get(), settings_c);
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
                new UIScale(1.0f),
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
        try (var properties = FxLib.INSTANCE.application_properties(PopcornFxInstance.INSTANCE.get())) {
            log.debug("Retrieved properties {}", properties);
            this.properties = properties;
        }
        try (var settings = FxLib.INSTANCE.application_settings(PopcornFxInstance.INSTANCE.get())) {
            log.debug("Retrieved settings {}", settings);
            this.settings = settings;
        }

        initializeDefaultLanguage();
        initializeSettings();

        FxLib.INSTANCE.register_settings_callback(PopcornFxInstance.INSTANCE.get(), callback);
    }

    private void initializeSettings() {
        var uiSettings = this.settings.getUiSettings();
        updateUIScale(uiSettings.getUiScale().getValue());
    }

    private void initializeDefaultLanguage() {
        //        var uiSettings = this.settings.getUiSettings();
        //
        //        // update the locale text with the locale from the settings
        //        localeText.updateLocale(uiSettings.getDefaultLanguage());
        //
        //        // add a listener to the default language for changing the language at runtime
        //        uiSettings.addListener(event -> {
        //            if (event.getPropertyName().equals(UISettings.LANGUAGE_PROPERTY)) {
        //                var locale = (Locale) event.getNewValue();
        //
        //                localeText.updateLocale(locale);
        //            }
        //        });
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

    private int getCurrentUIScaleIndex() {
        var uiSettings = settings.getUiSettings();
        var scale = uiSettings.getUiScale();
        var index = supportedUIScales().indexOf(scale);

        // check if the index was found
        // if not, return the index of the default
        if (index == -1) {
            log.warn("UI scale \"{}\" couldn't be found back in the supported UI scales", scale);
            index = supportedUIScales().indexOf(new UIScale(1.0f));
        }

        log.trace("Current UI scale index: {}", index);
        return index;
    }

    private void handleEvent(ApplicationConfigEvent.ByValue event) {
        if (event.tag == ApplicationConfigEvent.Tag.UiSettingsChanged) {
            var settings = event.getUnion().getUiSettings().getSettings();
            updateUIScale(settings.getUiScale().getValue());
        }
    }

    private ApplicationConfigEventCallback createCallback() {
        return event -> {
            try (event) {
                log.debug("Received settings event {}", event);
                handleEvent(event);
                for (var listener : listeners) {
                    listener.callback(event);
                }
            } catch (Exception ex) {
                log.error("Failed to invoke settings listener, {}", ex.getMessage(), ex);
            }
        };
    }

    //endregion
}
