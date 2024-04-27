package com.github.yoep.popcorn.backend.utils;

import java.util.Locale;
import java.util.ResourceBundle;

public interface LocaleText {

    /**
     * Get the resource bundle that is used by this {@link LocaleText}.
     *
     * @return Returns the resource bundle.
     */
    ResourceBundle getResourceBundle();

    /**
     * Get the text for the given message key.
     *
     * @param message Set the message key.
     * @return Returns the formatted text.
     */
    String get(Message message);

    /**
     * Get the text for the given message key.
     *
     * @param message Set the message key.
     * @param args    Set the arguments to pass to the message.
     * @return Returns the formatted text.
     */
    String get(Message message, Object... args);

    /**
     * Get the text for the given message.
     *
     * @param message Set the message.
     * @param args    Set the arguments to pass to the message.
     * @return Returns the formatted text.
     */
    String get(String message, Object... args);

    /**
     * Update the locale that needs to be used for the localized texts.
     *
     * @param locale The new locale to use.
     */
    void updateLocale(Locale locale);
}
