package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFxInstance;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.models.MovieOverview;
import lombok.extern.slf4j.Slf4j;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.PageImpl;
import org.springframework.stereotype.Service;

import java.util.Collections;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

@Slf4j
@Service
public class MovieProviderService implements ProviderService<MovieOverview> {
    private static final Category CATEGORY = Category.MOVIES;

    @Override
    public boolean supports(Category category) {
        return category == CATEGORY;
    }

    @Override
    public CompletableFuture<Page<MovieOverview>> getPage(Genre genre, SortBy sortBy, int page) {
        return CompletableFuture.completedFuture(getPage(genre, sortBy, "", page));
    }

    @Override
    public CompletableFuture<Page<MovieOverview>> getPage(Genre genre, SortBy sortBy, int page, String keywords) {
        return CompletableFuture.completedFuture(getPage(genre, sortBy, keywords, page));
    }

    @Override
    public CompletableFuture<MovieOverview> getDetails(String imdbId) {
        return CompletableFuture.completedFuture(getInternalDetails(imdbId));
    }

    @Override
    public CompletableFuture<Media> retrieveDetails(Media media) {
        return CompletableFuture.completedFuture(getInternalDetails(media.getId()));
    }

    @Override
    public void resetApiAvailability() {
        FxLib.INSTANCE.reset_movie_apis(PopcornFxInstance.INSTANCE.get());
    }

    public Page<MovieOverview> getPage(Genre genre, SortBy sortBy, String keywords, int page) {
        var movies = Optional.ofNullable(FxLib.INSTANCE.retrieve_available_movies(PopcornFxInstance.INSTANCE.get(), genre, sortBy, keywords, page))
                .map(MovieSet::getMovies)
                .orElse(Collections.emptyList());
        log.debug("Retrieved movies {}", movies);

        return new PageImpl<>(movies);
    }

    private static MovieDetails getInternalDetails(String imdbId) {
        var movie = FxLib.INSTANCE.retrieve_movie_details(PopcornFxInstance.INSTANCE.get(), imdbId);
        log.debug("Retrieved movie details {}", movie);

        return movie;
    }
}
