package com.github.yoep.popcorn.backend.media.providers;

import javax.validation.constraints.NotNull;
import java.net.URI;

public interface UriProvider {
    /**
     * Check if this uri provider can be used.
     *
     * @return Returns true if this provider can be used, else false.
     */
    boolean isAvailable();

    /**
     * Disable this provider as it's unavailable or failed.
     */
    void disable();

    /**
     * Reset the availability of this uri provider.
     * This will make the {@link #isAvailable()} return {@code true} even if the {@link #disable()} has been called.
     */
    void reset();

    /**
     * Get the uri of this provider.
     *
     * @return Returns the uri of this provider.
     */
    @NotNull
    URI getUri();
}
