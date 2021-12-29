package com.github.yoep.popcorn.backend.media.providers;

import lombok.AccessLevel;
import lombok.RequiredArgsConstructor;
import org.springframework.util.Assert;

import javax.validation.constraints.NotNull;
import java.net.URI;

@RequiredArgsConstructor(access = AccessLevel.PRIVATE)
public class DefaultUriProvider implements UriProvider {
    private final URI uri;

    private boolean disabled;

    //region Methods

    /**
     * Create a new default uri provider from the given uri.
     *
     * @param uri The uri this provider provides.
     * @return Returns the uri provider instance.
     */
    public static UriProvider from(URI uri) {
        Assert.notNull(uri, "uri cannot be null");
        return new DefaultUriProvider(uri);
    }

    //endregion

    //region UriProvider

    @Override
    public boolean isAvailable() {
        return !disabled;
    }

    @Override
    public void disable() {
        disabled = true;
    }

    @Override
    public @NotNull URI getUri() {
        return uri;
    }

    //endregion
}
