package com.github.yoep.popcorn.backend.utils;

import java.text.MessageFormat;
import java.util.Locale;
import java.util.ResourceBundle;

public class PopcornLocaleText implements LocaleText {
    private ResourceBundleMessageSource resourceBundle;

    public PopcornLocaleText(ResourceBundleMessageSource resourceBundle) {
        this.resourceBundle = resourceBundle;
    }

    @Override
    public ResourceBundle getResourceBundle() {
        return resourceBundle;
    }

    @Override
    public String get(Message message) {
        return get(message.getKey());
    }

    @Override
    public String get(Message message, Object... args) {
        return get(message.getKey(), args);
    }

    @Override
    public String get(String message, Object... args) {
        return MessageFormat.format(resourceBundle.getString(message), args);
    }

    @Override
    public void updateLocale(Locale locale) {
        resourceBundle.setLocale(locale);
    }
}
