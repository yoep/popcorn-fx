package com.github.yoep.popcorn.services;

import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.models.Genre;
import com.github.yoep.popcorn.models.SortBy;
import lombok.RequiredArgsConstructor;
import org.springframework.web.client.RestTemplate;
import org.springframework.web.util.UriComponentsBuilder;

import java.net.URI;

/**
 * Abstract implementation of {@link ProviderService}.
 *
 * @param <T> The media type this provider returns.
 */
@RequiredArgsConstructor
public abstract class AbstractProviderService<T extends Media> implements ProviderService<T> {
    protected final RestTemplate restTemplate;

    protected URI getUriFor(Genre genre, SortBy sortBy, String keywords, int page) {
        return UriComponentsBuilder.fromUri(getBaseUrl())
                .path("/{page}")
                .queryParam("sort", sortBy.getKey())
                .queryParam("order", -1)
                .queryParam("genre", genre.getKey())
                .queryParam("keywords", keywords)
                .build(page);
    }

    /**
     * Get the base url of the API to call.
     *
     * @return Returns the base url of the API to call.
     */
    protected abstract URI getBaseUrl();
}
