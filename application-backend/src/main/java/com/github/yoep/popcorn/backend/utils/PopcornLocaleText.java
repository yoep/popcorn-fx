package com.github.yoep.popcorn.backend.utils;

import lombok.ToString;
import lombok.extern.slf4j.Slf4j;

import java.text.MessageFormat;
import java.util.Locale;
import java.util.MissingResourceException;
import java.util.ResourceBundle;

@Slf4j
@ToString
public class PopcornLocaleText implements LocaleText {
    private final ResourceBundleMessageSource resourceBundle;

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
        try {
            var messageFormat = resourceBundle.getString(message);
            return MessageFormat.format(messageFormat, args);
        } catch (MissingResourceException ex) {
            log.error(ex.getMessage(), ex);
            return message;
        }
    }

    @Override
    public void updateLocale(Locale locale) {
        resourceBundle.setLocale(locale);
    }
}
