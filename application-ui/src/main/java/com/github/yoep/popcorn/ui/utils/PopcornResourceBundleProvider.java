package com.github.yoep.popcorn.ui.utils;

import lombok.extern.slf4j.Slf4j;

import java.io.IOException;
import java.io.InputStream;
import java.util.Locale;
import java.util.Optional;
import java.util.PropertyResourceBundle;
import java.util.ResourceBundle;
import java.util.spi.AbstractResourceBundleProvider;
import java.util.spi.ResourceBundleProvider;

@Slf4j
public class PopcornResourceBundleProvider extends AbstractResourceBundleProvider implements ResourceBundleProvider {
    public PopcornResourceBundleProvider() {
        super("java.properties");
    }

    @Override
    public ResourceBundle getBundle(String baseName, Locale locale) {
        // Construct bundle name for the given locale
        var bundleName = toBundleName(baseName, locale);
        var resourceName = toResourceName(bundleName, "properties");

        return Optional.ofNullable(PopcornResourceBundleProvider.class.getResourceAsStream(resourceName))
                .map(PopcornResourceBundleProvider::loadBundle)
                .orElseGet(() -> {
                    try {
                        return super.getBundle(resourceName, locale);
                    } catch (Exception e) {
                        log.error("Failed to load resource bundle: {}", bundleName, e);
                        return null;
                    }
                });
    }

    private static ResourceBundle loadBundle(InputStream is) {
        try {
            return new PropertyResourceBundle(is);
        } catch (IOException e) {
            log.error("Failed to load resource bundle", e);
            return null;
        }
    }

    private static String toResourceName(String bundleName, String suffix) {
        if (bundleName.contains("://")) {
            return null;
        }
        StringBuilder sb = new StringBuilder(bundleName.length() + 1 + suffix.length());
        sb.append(bundleName.replace('.', '/')).append('.').append(suffix);
        return sb.toString();
    }
}
