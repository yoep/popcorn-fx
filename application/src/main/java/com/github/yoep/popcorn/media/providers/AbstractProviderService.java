package com.github.yoep.popcorn.media.providers;

import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.models.Genre;
import com.github.yoep.popcorn.models.SortBy;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.web.client.RestTemplate;
import org.springframework.web.util.UriComponentsBuilder;

import java.net.URI;

/**
 * Abstract implementation of {@link ProviderService}.
 *
 * @param <T> The media type this provider returns.
 */
@Slf4j
@RequiredArgsConstructor
public abstract class AbstractProviderService<T extends Media> implements ProviderService<T> {
    protected final RestTemplate restTemplate;
    protected final ActivityManager activityManager;

    protected URI getUriFor(URI baseUrl, String resource, Genre genre, SortBy sortBy, String keywords, int page) {
        log.trace("Creating uri for base \"{}\", resource \"{}\", genre \"{}\", sort \"{}\", keywords \"{}\" and page \"{}\"",
                baseUrl, resource, genre, sortBy, keywords, page);
        return UriComponentsBuilder.fromUri(baseUrl)
                .path("/{resource}/{page}")
                .queryParam("sort", sortBy.getKey())
                .queryParam("order", -1)
                .queryParam("genre", genre.getKey())
                .queryParam("keywords", keywords)
                .build(resource, page);
    }
}
