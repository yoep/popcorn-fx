package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.media.MediaError;
import com.github.yoep.popcorn.backend.media.MediaSet;
import com.github.yoep.popcorn.backend.media.MediaSetResult;
import com.github.yoep.popcorn.backend.media.filters.model.Category;
import com.github.yoep.popcorn.backend.media.filters.model.Genre;
import com.github.yoep.popcorn.backend.media.filters.model.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.MovieDetails;
import com.github.yoep.popcorn.backend.media.providers.models.MovieOverview;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.PageImpl;
import org.springframework.stereotype.Service;

import java.util.Collections;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;

@Slf4j
@Service
@RequiredArgsConstructor
public class MovieProviderService implements ProviderService<MovieOverview> {
    private static final Category CATEGORY = Category.MOVIES;

    private final FxLib fxLib;
    private final PopcornFx instance;

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
        fxLib.reset_movie_apis(instance);
    }

    public Page<MovieOverview> getPage(Genre genre, SortBy sortBy, String keywords, int page) {
        try (var mediaResult = fxLib.retrieve_available_movies(instance, genre, sortBy, keywords, page)) {
            if (mediaResult.getTag() == MediaSetResult.Tag.Ok) {
                var movies = Optional.ofNullable(mediaResult.getUnion())
                        .map(MediaSetResult.MediaSetResultUnion::getOk)
                        .map(MediaSetResult.OkBody::getMediaSet)
                        .map(MediaSet::getMovies)
                        .orElse(Collections.emptyList());
                log.debug("Retrieved movies {}", movies);

                return new PageImpl<>(movies);
            } else {
                var mediaError = mediaResult.getUnion().getErr().getMediaError();
                if (mediaError == MediaError.NoAvailableProviders) {
                    throw new MediaRetrievalException(mediaError.getMessage());
                } else {
                    throw new MediaException(mediaError.getMessage());
                }
            }
        }
    }

    private MovieDetails getInternalDetails(String imdbId) {
        var movie = fxLib.retrieve_movie_details(instance, imdbId);
        log.debug("Retrieved movie details {}", movie);

        return movie;
    }
}
