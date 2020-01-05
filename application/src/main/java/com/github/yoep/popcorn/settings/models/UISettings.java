package com.github.yoep.popcorn.settings.models;

import lombok.*;

import java.util.Locale;
import java.util.Objects;

@EqualsAndHashCode(callSuper = false)
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class UISettings extends AbstractSettings {
    public static final String LANGUAGE_PROPERTY = "defaultLanguage";
    public static final String UI_SCALE_PROPERTY = "uiScale";

    private static final Locale DEFAULT_LANGUAGE = Locale.getDefault();

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
}
