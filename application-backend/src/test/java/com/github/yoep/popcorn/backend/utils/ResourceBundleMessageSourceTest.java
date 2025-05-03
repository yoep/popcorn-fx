package com.github.yoep.popcorn.backend.utils;

import org.junit.jupiter.api.Test;

import java.util.Locale;
import java.util.spi.AbstractResourceBundleProvider;

import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;

class ResourceBundleMessageSourceTest {
    @Test
    void testSetLocale() {
        var locale = Locale.FRENCH;
        var provider = mock(AbstractResourceBundleProvider.class);
        var messageSource = new ResourceBundleMessageSource(provider, "main");

        messageSource.setLocale(locale);

        verify(provider).getBundle(ResourceBundleMessageSource.RESOURCE_DIRECTORY + "main", locale);
    }
}