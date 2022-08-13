package com.github.yoep.popcorn.backend.settings.models;

import lombok.*;

import java.util.List;
import java.util.Locale;

import static java.util.Arrays.asList;

@EqualsAndHashCode(callSuper = false)
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class UISettings extends AbstractSettings {
    public static final String LANGUAGE_PROPERTY = "defaultLanguage";
    public static final String UI_SCALE_PROPERTY = "uiScale";
    public static final String START_SCREEN_PROPERTY = "startScreen";
    public static final String MAXIMIZED_PROPERTY = "maximized";
    public static final String NATIVE_WINDOW_PROPERTY = "useNativeWindow";

    public static final Locale DEFAULT_LANGUAGE = defaultLanguage();
    public static final UIScale DEFAULT_UI_SCALE = new UIScale(1f);

    /**
     * The default language of the application.
     */
    @Builder.Default
    private Locale defaultLanguage = DEFAULT_LANGUAGE;
    /**
     * The ui scale of the application.
     */
    @Builder.Default
    private UIScale uiScale = DEFAULT_UI_SCALE;
    /**
     * The default start screen of the application.
     */
    @Builder.Default
    private StartScreen startScreen = StartScreen.MOVIES;
    /**
     * The indication if the UI was maximized the last time the application was closed.
     */
    private boolean maximized;
    /**
     * The indication if the UI should use a native window rather than the borderless stage.
     */
    private boolean nativeWindowEnabled;

    //region Setters

    public void setDefaultLanguage(Locale defaultLanguage) {
        this.defaultLanguage = updateProperty(this.defaultLanguage, defaultLanguage, LANGUAGE_PROPERTY);
    }

    public void setUiScale(UIScale uiScale) {
        this.uiScale = updateProperty(this.uiScale, uiScale, UI_SCALE_PROPERTY);
    }

    public void setMaximized(boolean maximized) {
        this.maximized = updateProperty(this.maximized, maximized, MAXIMIZED_PROPERTY);
    }

    public void setNativeWindowEnabled(boolean nativeWindowEnabled) {
        this.nativeWindowEnabled = updateProperty(this.nativeWindowEnabled, nativeWindowEnabled, NATIVE_WINDOW_PROPERTY);
    }

    public void setStartScreen(StartScreen startScreen) {
        this.startScreen = updateProperty(this.startScreen, startScreen, START_SCREEN_PROPERTY);
    }

    //endregion

    /**
     * Get the default language for the application.
     *
     * @return Returns the default for the application.
     */
    public static Locale defaultLanguage() {
        var identifiedDefault = Locale.getDefault();

        return supportedLanguages().stream()
                .filter(e -> e.getLanguage().equals(identifiedDefault.getLanguage()))
                .findFirst()
                .orElse(Locale.ENGLISH);
    }

    /**
     * Get the list of supported languages by the application.
     *
     * @return Returns the list of support languages.
     */
    public static List<Locale> supportedLanguages() {
        return asList(
                Locale.ENGLISH,
                new Locale("nl"),
                new Locale("fr"));
    }
}
