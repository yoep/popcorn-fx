package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import com.github.yoep.popcorn.backend.config.properties.PopcornProperties;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.settings.SettingsService;
import lombok.extern.slf4j.Slf4j;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.PageImpl;
import org.springframework.stereotype.Service;
import org.springframework.web.client.RestTemplate;
import org.springframework.web.util.UriComponentsBuilder;

import java.text.MessageFormat;
import java.util.Collections;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

@Slf4j
@Service
public class MovieProviderService extends AbstractProviderService<Movie> {
    private static final Category CATEGORY = Category.MOVIES;

    public MovieProviderService(RestTemplate restTemplate,
                                PopcornProperties popcornConfig,
                                SettingsService settingsService) {
        super(restTemplate);

        initializeUriProviders(settingsService.getSettings().getServerSettings(), popcornConfig.getProvider(CATEGORY.getProviderName()));
    }

    @Override
    public boolean supports(Category category) {
        return category == CATEGORY;
    }

    @Override
    public CompletableFuture<Page<Movie>> getPage(Genre genre, SortBy sortBy, int page) {
        return CompletableFuture.completedFuture(getPage(genre, sortBy, "", page));
    }

    @Override
    public CompletableFuture<Page<Movie>> getPage(Genre genre, SortBy sortBy, int page, String keywords) {
        return CompletableFuture.completedFuture(getPage(genre, sortBy, keywords, page));
    }

    @Override
    public CompletableFuture<Movie> getDetails(String imdbId) {
        return invokeWithUriProvider(apiUri -> {
            var uri = UriComponentsBuilder.fromUri(apiUri)
                    .path("/movie/{id}")
                    .build(imdbId);

            log.debug("Retrieving movie details \"{}\"", uri);
            var response = restTemplate.getForEntity(uri, Movie.class);

            if (response.getBody() == null) {
                return CompletableFuture.failedFuture(new MediaException(MessageFormat.format("Failed to retrieve the details of {0}", imdbId)));
            }

            return CompletableFuture.completedFuture(response.getBody());
        });
    }

    @Override
    public CompletableFuture<Media> retrieveDetails(Media media) {
        // no additional details need to be loaded
        // so we'll return the media item directly
        return CompletableFuture.completedFuture(media);
    }

    public Page<Movie> getPage(Genre genre, SortBy sortBy, String keywords, int page) {
        var movies = Optional.ofNullable(FxLib.INSTANCE.retrieve_available_movies(PopcornFxInstance.INSTANCE.get(), genre, sortBy, keywords, page))
                .map(MovieSet::getMovies)
                .orElse(Collections.emptyList());
        log.debug("Retrieved movies {}", movies);

        return new PageImpl<>(movies);
    }
}
