package com.github.yoep.popcorn.media.providers;

import com.github.yoep.popcorn.config.properties.PopcornProperties;
import com.github.yoep.popcorn.config.properties.ProviderProperties;
import com.github.yoep.popcorn.media.providers.models.Movie;
import com.github.yoep.popcorn.models.Genre;
import com.github.yoep.popcorn.models.SortBy;
import org.apache.commons.lang3.StringUtils;
import org.springframework.http.ResponseEntity;
import org.springframework.stereotype.Component;
import org.springframework.web.client.RestTemplate;
import org.springframework.web.util.UriComponentsBuilder;

import java.net.URI;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;

@Component
public class MoviesProvider {
    private final RestTemplate restTemplate;
    private final ProviderProperties providerConfig;

    public MoviesProvider(RestTemplate restTemplate, PopcornProperties popcornConfig) {
        this.restTemplate = restTemplate;
        this.providerConfig = popcornConfig.getProvider("movies");
    }

    public List<Movie> getPage(Genre genre, SortBy sortBy, int page) {
        return getPage(genre, sortBy, StringUtils.EMPTY, page);
    }

    public List<Movie> getPage(Genre genre, SortBy sortBy, String keywords, int page) {
        URI uri = UriComponentsBuilder.fromUri(providerConfig.getUrl())
                .path("/{page}")
                .queryParam("sort", sortBy.getKey())
                .queryParam("order", -1)
                .queryParam("genre", genre.getKey())
                .queryParam("keywords", keywords)
                .build(page);

        ResponseEntity<Movie[]> movies = restTemplate.getForEntity(uri, Movie[].class);

        return Optional.ofNullable(movies.getBody())
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }
}
