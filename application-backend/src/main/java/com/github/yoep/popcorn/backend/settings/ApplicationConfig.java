package com.github.yoep.popcorn.backend.settings;

import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.ApplicationArgs;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.FxMessage;
import com.github.yoep.popcorn.backend.settings.models.*;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import lombok.extern.slf4j.Slf4j;

import java.util.*;
import java.util.concurrent.ConcurrentLinkedDeque;
import java.util.concurrent.ExecutionException;
import java.util.function.Consumer;

import static java.util.Arrays.asList;

@Slf4j
public class ApplicationConfig {
    private final FxChannel fxChannel;
    private final LocaleText localeText;

    private final Queue<ApplicationConfigEventCallback> listeners = new ConcurrentLinkedDeque<>();
    private final ApplicationConfigEventCallback callback = createCallback();

    private Consumer<Float> onUiScaleChanged;
    private ApplicationArgs applicationArgs;

    public ApplicationConfig(FxChannel fxChannel, LocaleText localeText) {
        Objects.requireNonNull(fxChannel, "fxChannel cannot be null");
        Objects.requireNonNull(localeText, "localeText cannot be null");
        this.fxChannel = fxChannel;
        this.localeText = localeText;
        init();
    }

    //region Properties

    /**
     * Get the application settings.
     *
     * @return Returns the application settings.
     */
    public ApplicationSettings getSettings() {
        return null;
    }

    public boolean isTvMode() {
        return applicationArgs().getIsTvMode();
    }

    public boolean isMaximized() {
        return applicationArgs().getIsMaximized();
    }

    public boolean isKioskMode() {
        return applicationArgs().getIsKioskMode();
    }

    public boolean isMouseDisabled() {
        return applicationArgs().getIsMouseDisabled();
    }

    public boolean isYoutubeVideoPlayerEnabled() {
        return applicationArgs().getIsYoutubePlayerEnabled();
    }

    public boolean isVlcVideoPlayerEnabled() {
        return applicationArgs().getIsVlcVideoPlayerEnabled();
    }

    public boolean isFxPlayerEnabled() {
        return applicationArgs().getIsFxPlayerEnabled();
    }

    public void setOnUiScaleChanged(Consumer<Float> onUiScaleChanged) {
        this.onUiScaleChanged = onUiScaleChanged;
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
        try (var settings_c = new SubtitleSettings.ByValue(settings)) {
//            fxLib.update_subtitle_settings(instance, settings_c);
        }
    }

    /**
     * Update the subtitle settings of the application with the new value.
     *
     * @param settings The new settings to use.
     */
    public void update(TorrentSettings settings) {
        Objects.requireNonNull(settings, "settings cannot be null");
        try (var settings_c = new TorrentSettings.ByValue(settings)) {
//            fxLib.update_torrent_settings(instance, settings_c);
        }
    }

    public void update(UISettings settings) {
        Objects.requireNonNull(settings, "settings cannot be null");
        var settings_c = new UISettings.ByValue(settings);
//        fxLib.update_ui_settings(instance, settings_c);
    }

    public void update(ServerSettings settings) {
        Objects.requireNonNull(settings, "settings cannot be null");
        var settings_c = new ServerSettings.ByValue(settings);
//        fxLib.update_server_settings(instance, settings_c);
    }

    public void update(PlaybackSettings settings) {
        Objects.requireNonNull(settings, "settings cannot be null");
        var settings_c = new PlaybackSettings.ByValue(settings);
//        fxLib.update_playback_settings(instance, settings_c);
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
                new UIScale(1.75f),
                new UIScale(2.0f),
                new UIScale(2.5f),
                new UIScale(3.0f),
                new UIScale(3.5f),
                new UIScale(4.0f),
                new UIScale(5.0f)
        );
    }

    //endregion

    //region PostConstruct

    private void init() {
        initializeSettings();
    }

    private ApplicationArgs applicationArgs() {
        if (applicationArgs == null) {
            try {
                this.applicationArgs = fxChannel.get(FxMessage.MessageType.APPLICATION_ARGS_REQUEST, ApplicationArgs.parser()).get();
            } catch (ExecutionException | InterruptedException e) {
                throw new RuntimeException(e);
            }
        }

        return applicationArgs;
    }

    private void initializeSettings() {
        var uiSettings = getSettings().getUiSettings();
        var locale = UISettings.supportedLanguages().stream()
                .filter(e -> e.getDisplayLanguage().equalsIgnoreCase(uiSettings.getDefaultLanguage()))
                .findFirst()
                .orElse(Locale.ENGLISH);

        updateUIScale(uiSettings.getUiScale().getValue());
        localeText.updateLocale(locale);
    }

    //endregion

    //region Functions

    private void updateUIScale(float scale) {
        Optional.ofNullable(onUiScaleChanged)
                .ifPresent(e -> e.accept(scale));
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
        if (event.tag == ApplicationConfigEvent.Tag.UI_SETTINGS_CHANGED) {
            var settings = event.getUnion().getUiSettingsChanged_body().getSettings();
            updateUIScale(settings.getUiScale().getValue());

            var language = settings.getDefaultLanguage();
            UISettings.supportedLanguages()
                    .stream()
                    .filter(e -> e.getDisplayLanguage().equalsIgnoreCase(language))
                    .findFirst()
                    .ifPresent(localeText::updateLocale);
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
