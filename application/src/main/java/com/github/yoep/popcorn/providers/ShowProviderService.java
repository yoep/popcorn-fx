package com.github.yoep.popcorn.providers;

import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.ShowSerieDetailsActivity;
import com.github.yoep.popcorn.config.properties.PopcornProperties;
import com.github.yoep.popcorn.config.properties.ProviderProperties;
import com.github.yoep.popcorn.models.Category;
import com.github.yoep.popcorn.models.Genre;
import com.github.yoep.popcorn.models.SortBy;
import com.github.yoep.popcorn.providers.models.Media;
import com.github.yoep.popcorn.providers.models.Show;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.http.ResponseEntity;
import org.springframework.stereotype.Service;
import org.springframework.web.client.RestTemplate;
import org.springframework.web.util.UriComponentsBuilder;

import java.net.URI;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

@Slf4j
@Service
public class ShowProviderService extends AbstractProviderService<Show> {
    private static final Category CATEGORY = Category.SERIES;
    private final ProviderProperties providerConfig;

    public ShowProviderService(RestTemplate restTemplate, ActivityManager activityManager, PopcornProperties popcornConfig) {
        super(restTemplate, activityManager);
        this.providerConfig = popcornConfig.getProvider(CATEGORY.getProviderName());
    }

    @Override
    public boolean supports(Category category) {
        return category == CATEGORY;
    }

    @Override
    public CompletableFuture<Show[]> getPage(Genre genre, SortBy sortBy, int page) {
        return CompletableFuture.completedFuture(getPage(genre, sortBy, StringUtils.EMPTY, page));
    }

    @Override
    public CompletableFuture<Show[]> getPage(Genre genre, SortBy sortBy, int page, String keywords) {
        return CompletableFuture.completedFuture(getPage(genre, sortBy, keywords, page));
    }

    //TODO: add UI feedback for the user if the API call fails
    @Override
    public void showDetails(Media media) {
        URI uri = UriComponentsBuilder.fromUri(providerConfig.getUrl())
                .path("{resource}/{imdb_id}")
                .build("show", media.getId());

        log.debug("Loading show details for \"{}\" IMDB ID", media.getId());
        ResponseEntity<Show> show = restTemplate.getForEntity(uri, Show.class);
        int statusCodeValue = show.getStatusCodeValue();

        if (statusCodeValue >= 200 && statusCodeValue < 300) {
            activityManager.register((ShowSerieDetailsActivity) show::getBody);
        } else {
            log.error("Failed to load the show details with \"{}\" status", show.getStatusCode());
        }
    }

    public Show[] getPage(Genre genre, SortBy sortBy, String keywords, int page) {
        URI uri = getUriFor(providerConfig.getUrl(), "shows", genre, sortBy, keywords, page);

        log.debug("Retrieving show provider page \"{}\"", uri);
        ResponseEntity<Show[]> shows = restTemplate.getForEntity(uri, Show[].class);

        return Optional.ofNullable(shows.getBody())
                .orElse(new Show[0]);
    }
}
