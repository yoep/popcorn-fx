package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.config.properties.ProviderProperties;
import com.github.yoep.popcorn.backend.media.filters.models.Genre;
import com.github.yoep.popcorn.backend.media.filters.models.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.settings.models.ServerSettings;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.data.domain.PageImpl;
import org.springframework.data.domain.Pageable;
import org.springframework.http.converter.HttpMessageNotReadableException;
import org.springframework.util.Assert;
import org.springframework.web.client.RestClientException;
import org.springframework.web.client.RestTemplate;
import org.springframework.web.util.UriComponentsBuilder;

import java.net.URI;
import java.text.MessageFormat;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.function.Function;
import java.util.stream.Collectors;

/**
 * Abstract implementation of {@link ProviderService}.
 *
 * @param <T> The media type this provider returns.
 */
@Slf4j
@RequiredArgsConstructor
public abstract class AbstractProviderService<T extends Media> implements ProviderService<T> {
    /**
     * The max. items returned in 1 page.
     * This value is according to the API documentation.
     */
    public static final int MAX_ITEMS = 50;

    protected final RestTemplate restTemplate;

    private final List<UriProvider> uriProviders = new ArrayList<>();

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

    protected void initializeUriProviders(ServerSettings serverSettings, ProviderProperties providerConfig) {
        Assert.notNull(serverSettings, "serverSettings cannot be null");
        Assert.notNull(providerConfig, "providerConfig cannot be null");

        // subscribe to the server settings for potential api server changes
        serverSettings.addListener(evt -> {
            if (evt.getPropertyName().equals(ServerSettings.API_SERVER_PROPERTY)) {
                // rebuild the uri provider list
                buildUriProvidersList(serverSettings, providerConfig);
            }
        });

        buildUriProvidersList(serverSettings, providerConfig);
    }

    /**
     * Invoke the given action with the available uri providers.
     * The action will be executed until it succeeds with {@link UriProvider} or until the providers are exhausted.
     *
     * @param action The action to execute.
     * @param <R>    The outcome/result of the action that is being executed.
     * @return Returns the result of the action with the succeeded uri provider.
     * @throws MediaException Is thrown when the action couldn't be successfully invoked with any available uri provider.
     *                        This most of the time means that all available API servers are down and this {@link ProviderService} cannot provide any
     *                        {@link Media} items.
     */
    protected <R> R invokeWithUriProvider(Function<URI, R> action) {
        UriProvider provider;

        do {
            provider = getUriProvider();

            try {
                return action.apply(provider.getUri());
            } catch (RestClientException ex) {
                handleException(provider, ex);
            }
        } while (true);
    }

    protected PageImpl<T> emptyPage() {
        return new PageImpl<>(Collections.emptyList(), Pageable.unpaged(), 0);
    }

    private void handleException(UriProvider provider, RestClientException ex) {
        if (ex.getCause() instanceof HttpMessageNotReadableException) {
            handleInvalidResponseException(provider, ex);
        } else {
            handleProviderException(provider, ex);
        }
    }

    private void handleInvalidResponseException(UriProvider provider, RestClientException ex) {
        log.error("Failed to parse API response, {}", ex.getMessage(), ex);
        throw new MediaParsingException(provider.getUri(), "Failed to parse API response", ex);
    }

    private void handleProviderException(UriProvider provider, RestClientException ex) {
        var message = MessageFormat.format("URI provider failed with error \"{0}\", using next uri provider", ex.getMessage());
        log.warn(message, ex);
        provider.disable();
    }

    private void buildUriProvidersList(ServerSettings serverSettings, ProviderProperties providerConfig) {
        synchronized (uriProviders) {
            // clear the list in case it is being rebuild
            uriProviders.clear();

            // check if a custom api server is configured
            // if so, add it to the uri providers list as the first item
            serverSettings.getApiServer()
                    .map(DefaultUriProvider::from)
                    .ifPresent(uriProviders::add);

            // add all available uri from the provider config to the uri providers
            var newUriProviders = providerConfig.getUris().stream()
                    .map(DefaultUriProvider::from)
                    .collect(Collectors.toList());

            uriProviders.addAll(newUriProviders);
        }
    }

    /**
     * Get the first available uri provider.
     *
     * @return Returns the first available uri provider.
     * @throws MediaException Is thrown when no uri providers are available.
     */
    private UriProvider getUriProvider() {
        synchronized (uriProviders) {
            return uriProviders.stream()
                    .filter(UriProvider::isAvailable)
                    .findFirst()
                    .orElseThrow(() -> new MediaException("No uri providers are available to be used"));
        }
    }
}
