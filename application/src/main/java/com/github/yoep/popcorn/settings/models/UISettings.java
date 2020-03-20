package com.github.yoep.popcorn.settings.models;

import lombok.*;

import java.util.List;
import java.util.Locale;
import java.util.Objects;

import static java.util.Arrays.asList;

@EqualsAndHashCode(callSuper = false)
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class UISettings extends AbstractSettings {
    public static final String LANGUAGE_PROPERTY = "defaultLanguage";
    public static final String UI_SCALE_PROPERTY = "uiScale";
    public static final String MAXIMIZED_PROPERTY = "maximized";

    private static final Locale DEFAULT_LANGUAGE = defaultLanguage();

    /**
     * The default language of the application.
     */
    @Builder.Default
    private Locale defaultLanguage = DEFAULT_LANGUAGE;
    /**
     * The ui scale of the application.
     */
    @Builder.Default
    private UIScale uiScale = new UIScale(1f);
    /**
     * The indication if the UI was maximized the last time the application was closed.
     */
    private boolean maximized;

    //region Setters

    public void setDefaultLanguage(Locale defaultLanguage) {
        if (Objects.equals(this.defaultLanguage, defaultLanguage))
            return;

        var oldValue = this.defaultLanguage;
        this.defaultLanguage = defaultLanguage;
        changes.firePropertyChange(LANGUAGE_PROPERTY, oldValue, defaultLanguage);
    }

    public void setUiScale(UIScale uiScale) {
        if (Objects.equals(this.uiScale, uiScale))
            return;

        var oldValue = this.uiScale;
        this.uiScale = uiScale;
        changes.firePropertyChange(UI_SCALE_PROPERTY, oldValue, uiScale);
    }

    public void setMaximized(boolean maximized) {
        if (Objects.equals(this.maximized, maximized))
            return;

        var oldValue = this.maximized;
        this.maximized = maximized;
        changes.firePropertyChange(MAXIMIZED_PROPERTY, oldValue, maximized);
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
        return asList(Locale.ENGLISH, new Locale("nl"));
    }
}
