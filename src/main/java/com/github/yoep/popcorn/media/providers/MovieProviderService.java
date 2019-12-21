package com.github.yoep.popcorn.media.providers;

import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.ShowMovieDetailsActivity;
import com.github.yoep.popcorn.config.properties.PopcornProperties;
import com.github.yoep.popcorn.config.properties.ProviderProperties;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Movie;
import com.github.yoep.popcorn.models.Category;
import com.github.yoep.popcorn.models.Genre;
import com.github.yoep.popcorn.models.SortBy;
import com.github.yoep.popcorn.subtitle.SubtitleService;
import org.apache.logging.log4j.util.Strings;
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
public class MovieProviderService extends AbstractProviderService<Movie> {
    private static final Category CATEGORY = Category.MOVIES;
    private final ProviderProperties providerConfig;
    private final SubtitleService subtitleService;

    public MovieProviderService(RestTemplate restTemplate, ActivityManager activityManager, PopcornProperties popcornConfig, SubtitleService subtitleService) {
        super(restTemplate, activityManager);
        this.providerConfig = popcornConfig.getProvider(CATEGORY.getProviderName());
        this.subtitleService = subtitleService;
    }

    @Override
    public boolean supports(Category category) {
        return category == CATEGORY;
    }

    @Override
    public CompletableFuture<List<Movie>> getPage(Genre genre, SortBy sortBy, int page) {
        return CompletableFuture.completedFuture(getPage(genre, sortBy, Strings.EMPTY, page));
    }

    @Override
    public CompletableFuture<List<Movie>> getPage(Genre genre, SortBy sortBy, int page, String keywords) {
        return CompletableFuture.completedFuture(getPage(genre, sortBy, keywords, page));
    }

    @Override
    public void showDetails(Media media) {
        final Movie movie = (Movie) media;

        activityManager.register((ShowMovieDetailsActivity) () -> movie);
        subtitleService.retrieveSubtitles(movie);
    }

    public List<Movie> getPage(Genre genre, SortBy sortBy, String keywords, int page) {
        URI uri = getUriFor("movies", genre, sortBy, keywords, page);

        ResponseEntity<Movie[]> items = restTemplate.getForEntity(uri, Movie[].class);

        return Optional.ofNullable(items.getBody())
                .map(Arrays::asList)
                .orElse(Collections.emptyList());
    }

    @Override
    protected URI getBaseUrl() {
        return providerConfig.getUrl();
    }
}
