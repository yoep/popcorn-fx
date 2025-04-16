package com.github.yoep.popcorn.backend.settings;

import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.FxChannelException;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import javafx.application.Platform;
import lombok.extern.slf4j.Slf4j;

import java.util.*;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ConcurrentLinkedDeque;
import java.util.concurrent.ExecutionException;
import java.util.function.Consumer;

import static java.util.Arrays.asList;

@Slf4j
public class ApplicationConfig {
    private final FxChannel fxChannel;
    private final LocaleText localeText;

    private final Queue<ApplicationSettingsEvent> listeners = new ConcurrentLinkedDeque<>();

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
    public CompletableFuture<ApplicationSettings> getSettings() {
        return fxChannel.send(ApplicationSettingsRequest.getDefaultInstance(), ApplicationSettingsResponse.parser())
                .thenApply(ApplicationSettingsResponse::getSettings);
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
        changeScale(1);
    }

    /**
     * Decrease the current UI scale.
     */
    public void decreaseUIScale() {
        changeScale(-1);
    }

    public void register(ApplicationSettingsEvent callback) {
        Objects.requireNonNull(callback, "callback cannot be null");
        listeners.add(callback);
    }

    /**
     * Update the subtitle settings of the application with the new value.
     *
     * @param settings The new settings to use.
     */
    public void update(ApplicationSettings.SubtitleSettings settings) {
        Objects.requireNonNull(settings, "settings cannot be null");
        fxChannel.send(UpdateSubtitleSettingsRequest.newBuilder()
                .setSettings(settings)
                .build());
    }

    /**
     * Update the subtitle settings of the application with the new value.
     *
     * @param settings The new settings to use.
     */
    public void update(ApplicationSettings.TorrentSettings settings) {
        Objects.requireNonNull(settings, "settings cannot be null");
        fxChannel.send(UpdateTorrentSettingsRequest.newBuilder()
                .setSettings(settings)
                .build());
    }

    public void update(ApplicationSettings.UISettings settings) {
        Objects.requireNonNull(settings, "settings cannot be null");
        fxChannel.send(UpdateUISettingsRequest.newBuilder()
                .setSettings(settings)
                .build());
    }

    public void update(ApplicationSettings.ServerSettings settings) {
        Objects.requireNonNull(settings, "settings cannot be null");
        fxChannel.send(UpdateServerSettingsRequest.newBuilder()
                .setSettings(settings)
                .build());
    }

    public void update(ApplicationSettings.PlaybackSettings settings) {
        Objects.requireNonNull(settings, "settings cannot be null");
        fxChannel.send(UpdatePlaybackSettingsRequest.newBuilder()
                .setSettings(settings)
                .build());
    }

    /**
     * Get the list of supported UI scales for this application.
     *
     * @return Returns a list of supported UI scales.
     */
    public static List<ApplicationSettings.UISettings.Scale> supportedUIScales() {
        return asList(
                ApplicationSettings.UISettings.Scale.newBuilder().setFactor(0.25f).build(),
                ApplicationSettings.UISettings.Scale.newBuilder().setFactor(0.5f).build(),
                ApplicationSettings.UISettings.Scale.newBuilder().setFactor(0.75f).build(),
                ApplicationSettings.UISettings.Scale.newBuilder().setFactor(1.0f).build(),
                ApplicationSettings.UISettings.Scale.newBuilder().setFactor(1.25f).build(),
                ApplicationSettings.UISettings.Scale.newBuilder().setFactor(1.50f).build(),
                ApplicationSettings.UISettings.Scale.newBuilder().setFactor(1.75f).build(),
                ApplicationSettings.UISettings.Scale.newBuilder().setFactor(2.0f).build(),
                ApplicationSettings.UISettings.Scale.newBuilder().setFactor(2.5f).build(),
                ApplicationSettings.UISettings.Scale.newBuilder().setFactor(3.0f).build(),
                ApplicationSettings.UISettings.Scale.newBuilder().setFactor(3.5f).build(),
                ApplicationSettings.UISettings.Scale.newBuilder().setFactor(4.0f).build(),
                ApplicationSettings.UISettings.Scale.newBuilder().setFactor(5.0f).build()
        );
    }

    /**
     * Get the supported UI languages.
     *
     * @return Returns the supported languages.
     */
    public static List<Locale> supportedLanguages() {
        return asList(
                Locale.ENGLISH,
                new Locale("nl"),
                new Locale("fr"));
    }

    //endregion

    //region PostConstruct

    private void init() {
        initializeSettings();
    }

    private ApplicationArgs applicationArgs() {
        if (applicationArgs == null) {
            try {
                this.applicationArgs = fxChannel.send(ApplicationArgsRequest.getDefaultInstance(), ApplicationArgsResponse.parser())
                        .thenApply(ApplicationArgsResponse::getArgs)
                        .get();
            } catch (ExecutionException | InterruptedException ex) {
                throw new FxChannelException(ex.getMessage(), ex);
            }
        }

        return applicationArgs;
    }

    private void initializeSettings() {
        getSettings().whenComplete((settings, throwable) -> {
            if (throwable == null) {
                var uiSettings = settings.getUiSettings();
                var locale = supportedLanguages().stream()
                        .filter(e -> e.getDisplayLanguage().equalsIgnoreCase(uiSettings.getDefaultLanguage()))
                        .findFirst()
                        .orElse(Locale.ENGLISH);

                Platform.runLater(() -> {
                    updateUIScale(uiSettings.getScale().getFactor());
                    localeText.updateLocale(locale);
                });
            } else {
                log.error("Failed to retrieve settings", throwable);
            }
        });
    }

    //endregion

    //region Functions

    private void changeScale(int indexChange) {
        var supportedUIScales = supportedUIScales();
        getCurrentUIScaleIndex().whenComplete((currentIndex, throwable) -> {
            if (throwable == null) {
                var newIndex = currentIndex + indexChange;

                // verify that the current UI scale is within the supported scales
                if (newIndex == supportedUIScales.size() - 1 || newIndex < 0)
                    return;

                getSettings().whenComplete((settings, ex) -> {
                    if (ex == null) {
                        update(ApplicationSettings.UISettings.newBuilder(settings.getUiSettings())
                                .setScale(supportedUIScales.get(newIndex))
                                .build());
                    } else {
                        log.error("Failed to retrieve settings", ex);
                    }
                });
            } else {
                log.error("Failed to retrieve settings", throwable);
            }
        });
    }

    private void updateUIScale(float scale) {
        Optional.ofNullable(onUiScaleChanged)
                .ifPresent(e -> e.accept(scale));
    }

    private CompletableFuture<Integer> getCurrentUIScaleIndex() {
        return getSettings().thenApply(response -> {
            var uiSettings = response.getUiSettings();
            var scale = uiSettings.getScale();
            var index = supportedUIScales().indexOf(scale);

            // check if the index was found
            // if not, return the index of the default
            if (index == -1) {
                log.warn("UI scale \"{}\" couldn't be found back in the supported UI scales", scale);
                index = supportedUIScales().indexOf(ApplicationSettings.UISettings.Scale.newBuilder()
                        .setFactor(1.0f)
                        .build());
            }

            log.trace("Current UI scale index: {}", index);
            return index;
        });
    }

    //endregion
}
