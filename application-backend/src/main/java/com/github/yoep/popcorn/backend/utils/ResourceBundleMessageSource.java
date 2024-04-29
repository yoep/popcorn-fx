package com.github.yoep.popcorn.backend.utils;

import lombok.Getter;
import lombok.extern.slf4j.Slf4j;

import java.util.*;
import java.util.spi.ResourceBundleProvider;

@Slf4j
@Getter
public class ResourceBundleMessageSource extends ResourceBundle {
    static final String RESOURCE_DIRECTORY = "/lang/";
    static final Locale DEFAULT_LOCAL = Locale.ENGLISH;
    private final Map<Locale, List<ResourceBundle>> resourceBundles = new HashMap<>();
    private final ResourceBundleProvider resourceBundleProvider;

    private Locale currentLocale = Locale.ENGLISH;
    private String[] basenames;

    public ResourceBundleMessageSource(ResourceBundleProvider resourceBundleProvider, String... basenames) {
        Objects.requireNonNull(resourceBundleProvider, "resourceBundleProvider cannot be null");
        this.basenames = basenames;
        this.resourceBundleProvider = resourceBundleProvider;
        doGetResourceBundle(currentLocale);
    }

    public void setLocale(Locale locale) {
        Objects.requireNonNull(locale, "locale cannot be null");
        if (locale.equals(currentLocale)) {
            return;
        }

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
                bundles.add(resourceBundleProvider.getBundle(RESOURCE_DIRECTORY + basename, locale));
            }

            resourceBundles.put(locale, bundles);
        }
    }
}
