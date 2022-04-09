package com.github.yoep.popcorn.backend.media.providers;

import org.junit.jupiter.api.Test;

import java.net.URI;

import static org.junit.jupiter.api.Assertions.*;

class DefaultUriProviderTest {
    private final URI uri = URI.create("http://localhost:8754/api");

    @Test
    void testIsAvailable_whenDisabledHasNotBeenInvoked_shouldReturnTrue() {
        var provider = DefaultUriProvider.from(uri);

        var result = provider.isAvailable();

        assertTrue(result, "Expected the provider to be available");
    }

    @Test
    void testIsAvailable_whenDisabledHasBeenInvoked_shouldReturnFalse() {
        var provider = DefaultUriProvider.from(uri);

        provider.disable();
        var result = provider.isAvailable();

        assertFalse(result, "Expected the provider to be unavailable");
    }

    @Test
    void testIsAvailable_whenDisabledAndResetHasBeenInvoked_shouldReturnTrue() {
        var provider = DefaultUriProvider.from(uri);

        provider.disable();
        provider.reset();
        var result = provider.isAvailable();

        assertTrue(result, "Expected the provider to be available");
    }

    @Test
    void testGetUri_whenInvoked_shouldReturnTheUriOfTheProvider() {
        var provider = DefaultUriProvider.from(uri);

        var result = provider.getUri();

        assertEquals(uri, result);
    }
}