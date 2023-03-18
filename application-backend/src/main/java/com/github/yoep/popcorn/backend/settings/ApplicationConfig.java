package com.github.yoep.popcorn.backend.settings;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.settings.models.*;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.util.List;
import java.util.Locale;
import java.util.Objects;
import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedDeque;

import static java.util.Arrays.asList;

@Slf4j
@Service
@RequiredArgsConstructor
public class ApplicationConfig {
    private final ViewLoader viewLoader;
    private final LocaleText localeText;
    private final FxLib fxLib;
    private final PopcornFx instance;

    private final Queue<ApplicationConfigEventCallback> listeners = new ConcurrentLinkedDeque<>();
    private final ApplicationConfigEventCallback callback = createCallback();

    private ApplicationSettings cachedSettings;

    //region Getters

    /**
     * Get the application settings.
     *
     * @return Returns the application settings.
     */
    public ApplicationSettings getSettings() {
        if (cachedSettings == null) {
            try (var settings = fxLib.application_settings(instance)) {
                log.debug("Retrieved settings {}", settings);
                cachedSettings = settings;
            }
        }

        return cachedSettings;
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

        var settings = getSettings().getUiSettings();
        settings.setUiScale(supportedUIScales.get(index + 1));
        update(settings);
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

        var settings = getSettings().getUiSettings();
        settings.setUiScale(supportedUIScales.get(index - 1));
        update(settings);
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
        fxLib.update_subtitle_settings(instance, settings_c);
    }

    /**
     * Update the subtitle settings of the application with the new value.
     *
     * @param settings The new settings to use.
     */
    public void update(TorrentSettings settings) {
        Objects.requireNonNull(settings, "settings cannot be null");
        var settings_c = new TorrentSettings.ByValue(settings);
        fxLib.update_torrent_settings(instance, settings_c);
    }

    public void update(UISettings settings) {
        Objects.requireNonNull(settings, "settings cannot be null");
        var settings_c = new UISettings.ByValue(settings);
        fxLib.update_ui_settings(instance, settings_c);
    }

    public void update(ServerSettings settings) {
        Objects.requireNonNull(settings, "settings cannot be null");
        var settings_c = new ServerSettings.ByValue(settings);
        fxLib.update_server_settings(instance, settings_c);
    }

    public void update(PlaybackSettings settings) {
        Objects.requireNonNull(settings, "settings cannot be null");
        var settings_c = new PlaybackSettings.ByValue(settings);
        fxLib.update_playback_settings(instance, settings_c);
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
        initializeSettings();
        fxLib.register_settings_callback(instance, callback);
    }

    private void initializeSettings() {
        var uiSettings = getSettings().getUiSettings();
        updateUIScale(uiSettings.getUiScale().getValue());
        localeText.updateLocale(Locale.forLanguageTag(uiSettings.getDefaultLanguage()));
    }

    //endregion

    //region Functions

    private void updateUIScale(float scale) {
        viewLoader.setScale(scale);
    }

    private int getCurrentUIScaleIndex() {
        var uiSettings = getSettings().getUiSettings();
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
            localeText.updateLocale(Locale.forLanguageTag(settings.getDefaultLanguage()));
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
