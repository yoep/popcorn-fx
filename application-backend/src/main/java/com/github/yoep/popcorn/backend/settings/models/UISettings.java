package com.github.yoep.popcorn.backend.settings.models;

import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.sun.jna.Structure;
import lombok.*;

import java.io.Closeable;
import java.util.List;
import java.util.Locale;
import java.util.Objects;

import static java.util.Arrays.asList;

@EqualsAndHashCode(callSuper = false)
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
@Structure.FieldOrder({"defaultLanguage", "uiScale", "startScreen", "maximized", "nativeWindowEnabled"})
public class UISettings extends Structure implements Closeable {
    public static class ByValue extends UISettings implements Structure.ByValue {
        public ByValue() {
        }

        public ByValue(UISettings settings) {
            Objects.requireNonNull(settings, "settings cannot be null");
            this.defaultLanguage = settings.defaultLanguage;
            this.uiScale = settings.uiScale;
            this.startScreen = settings.startScreen;
            this.maximized = settings.maximized;
            this.nativeWindowEnabled = settings.nativeWindowEnabled;
        }
    }

    public String defaultLanguage;
    public UIScale uiScale;
    public Category startScreen;
    public byte maximized;
    public byte nativeWindowEnabled;

    //region Methods

    public boolean isMaximized() {
        return maximized == 1;
    }

    public void setMaximized(boolean maximized) {
        this.maximized = (byte) (maximized ? 1 : 0);
    }

    public boolean isNativeWindowEnabled() {
        return nativeWindowEnabled == 1;
    }

    public void setNativeWindowEnabled(boolean nativeWindowEnabled) {
        this.nativeWindowEnabled = (byte) (nativeWindowEnabled ? 1 : 0);
    }

    @Override
    public void close() {
        setAutoSynch(false);
        uiScale.close();
    }

    //endregion

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
