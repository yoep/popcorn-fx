package com.github.yoep.popcorn.backend.settings.models;

import org.junit.jupiter.api.Test;

import java.util.Locale;

import static org.junit.jupiter.api.Assertions.assertEquals;

class UISettingsTest extends AbstractPropertyTest<UISettings> {
    UISettingsTest() {
        super(UISettings.class);
    }

    @Test
    void testSetDefaultLanguage_whenValueIsDifferent_shouldNotifyPropertyChanged() {
        var newValue = Locale.CANADA;

        settings.setDefaultLanguage(newValue);

        assertEquals(UISettings.LANGUAGE_PROPERTY, invokedPropertyName());
        assertEquals(newValue, invokedNewValue());
    }

    @Test
    void testSetUiScale_whenValueIsDifferent_shouldNotifyPropertyChanged() {
        var newValue = new UIScale(2f);

        settings.setUiScale(newValue);

        assertEquals(UISettings.UI_SCALE_PROPERTY, invokedPropertyName());
        assertEquals(newValue, invokedNewValue());
    }

    @Test
    void testSetMaximized_whenValueIsDifferent_shouldNotifyPropertyChanged() {
        var newValue = true;

        settings.setMaximized(newValue);

        assertEquals(UISettings.MAXIMIZED_PROPERTY, invokedPropertyName());
        assertEquals(newValue, invokedNewValue());
    }

    @Test
    void testSetNativeWindowEnabled_whenValueIsDifferent_shouldNotifyPropertyChanged() {
        var newValue = true;

        settings.setNativeWindowEnabled(newValue);

        assertEquals(UISettings.NATIVE_WINDOW_PROPERTY, invokedPropertyName());
        assertEquals(newValue, invokedNewValue());
    }

    @Test
    void testSetStartScreen_whenValueIsDifferent_shouldNotifyPropertyChanged() {
        var newValue = StartScreen.FAVORITES;

        settings.setStartScreen(newValue);

        assertEquals(UISettings.START_SCREEN_PROPERTY, invokedPropertyName());
        assertEquals(newValue, invokedNewValue());
    }
}