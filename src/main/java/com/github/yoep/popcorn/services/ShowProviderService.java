package com.github.yoep.popcorn.services;

import com.github.yoep.popcorn.config.properties.PopcornProperties;
import com.github.yoep.popcorn.config.properties.ProviderProperties;
import com.github.yoep.popcorn.media.providers.models.Show;
import com.github.yoep.popcorn.models.Category;
import com.github.yoep.popcorn.models.Genre;
import com.github.yoep.popcorn.models.SortBy;
import org.apache.commons.lang3.StringUtils;
import org.springframework.http.ResponseEntity;
import org.springframework.stereotype.Service;
import org.springframework.web.client.RestTemplate;

import java.net.URI;
import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

@Service
public class ShowProviderService extends AbstractProviderService<Show> {
    private static final Category CATEGORY = Category.SERIES;
    private final ProviderProperties providerConfig;

    public ShowProviderService(RestTemplate restTemplate, PopcornProperties popcornConfig) {
        super(restTemplate);
        this.providerConfig = popcornConfig.getProvider(CATEGORY.getProviderName());
    }

    @Override
    public boolean supports(Category category) {
        return category == CATEGORY;
    }

    @Override
    public CompletableFuture<List<Show>> getPage(Genre genre, SortBy sortBy, int page) {
        return CompletableFuture.completedFuture(getPage(genre, sortBy, StringUtils.EMPTY, page));
    }

    public List<Show> getPage(Genre genre, SortBy sortBy, String keywords, int page) {
        URI uri = getUriFor(genre, sortBy, keywords, page);

        ResponseEntity<Show[]> shows = restTemplate.getForEntity(uri, Show[].class);

        return Optional.ofNullable(shows.getBody())
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    @Override
    protected URI getBaseUrl() {
        return providerConfig.getUrl();
    }
}
