package com.github.yoep.popcorn.backend.utils;

import lombok.Getter;
import lombok.extern.slf4j.Slf4j;

import java.io.IOException;
import java.io.InputStream;
import java.util.*;

@Slf4j
@Getter
public class ResourceBundleMessageSource extends ResourceBundle {
    static final String RESOURCE_DIRECTORY = "/lang/";
    static final Locale DEFAULT_LOCAL = Locale.ENGLISH;
    private final Map<Locale, List<ResourceBundle>> resourceBundles = new HashMap<>();

    private Locale currentLocale = Locale.ENGLISH;
    private String[] basenames;

    public ResourceBundleMessageSource(String... basenames) {
        this.basenames = basenames;
        doGetResourceBundle(currentLocale);
    }

    public void setLocale(Locale locale) {
        this.currentLocale = locale;
        doGetResourceBundle(locale);
    }

    @Override
    protected Object handleGetObject(String key) {
        return getBundleWithKey(key)
                .map(e -> e.getObject(key))
                .orElse(null);
    }

    @Override
    public Enumeration<String> getKeys() {
        return Collections.enumeration(getAllKeys());
    }

    private Optional<ResourceBundle> getBundleWithKey(String key) {
        var bundles = resourceBundles.get(currentLocale);

        return bundles.stream()
                .filter(e -> getKeysFrom(e).contains(key))
                .findFirst();
    }

    private List<String> getAllKeys() {
        var keys = new ArrayList<String>();
        var bundles = resourceBundles.get(currentLocale);

        for (var bundle : bundles) {
            keys.addAll(getKeysFrom(bundle));
        }

        return keys;
    }

    private List<String> getKeysFrom(ResourceBundle bundle) {
        var keys = new ArrayList<String>();
        var iterator = bundle.getKeys().asIterator();

        while (iterator.hasNext()) {
            keys.add(iterator.next());
        }

        return keys;
    }

    private void doGetResourceBundle(Locale locale) {
        if (!resourceBundles.containsKey(locale)) {
            log.debug("Loading resource bundle for locale: {}", locale);
            var bundles = new ArrayList<ResourceBundle>();

            for (String basename : this.basenames) {
                bundles.add(ResourceBundle.getBundle(RESOURCE_DIRECTORY + basename, locale, new Control()));
            }

            resourceBundles.put(locale, bundles);
        }
    }

    private static ResourceBundle loadBundle(InputStream is) {
        try {
            return new PropertyResourceBundle(is);
        } catch (IOException e) {
            log.error("Failed to load resource bundle", e);
            return null;
        }
    }

    private static class Control extends ResourceBundle.Control {
        private static final Locale DEFAULT_LOCALE = Locale.ENGLISH;

        @Override
        public ResourceBundle newBundle(String baseName, Locale locale, String format, ClassLoader loader, boolean reload)
                throws IllegalAccessException, InstantiationException, java.io.IOException {
            // Special handling of default encoding
            if (format.equals("java.properties")) {
                // Construct bundle name for the given locale
                var bundleName = toBundleName(baseName, locale);
                var resourceName = toResourceName(bundleName, "properties");

                return Optional.ofNullable(loader.getResourceAsStream(resourceName))
                        .or(() -> Optional.ofNullable(Control.class.getResourceAsStream(resourceName)))
                        .map(ResourceBundleMessageSource::loadBundle)
                        .orElseGet(() -> {
                            try {
                                return super.newBundle(resourceName, locale, format, loader, reload);
                            } catch (Exception e) {
                                log.error("Failed to load resource bundle: {}", bundleName, e);
                                return null;
                            }
                        });
            } else {
                // Delegate handling of "java.class" format to standard Control
                return super.newBundle(baseName, locale, format, loader, reload);
            }
        }

        @Override
        public Locale getFallbackLocale(String baseName, Locale locale) {
            return (DEFAULT_LOCALE != locale) ? DEFAULT_LOCALE : null;
        }
    }
}
