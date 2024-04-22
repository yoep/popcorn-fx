package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.PopcornFx;
import com.github.yoep.popcorn.backend.media.MediaItem;
import com.github.yoep.popcorn.backend.media.MediaResult;
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

import java.util.Collections;
import java.util.List;
import java.util.Optional;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutorService;

@Slf4j
@RequiredArgsConstructor
public class MovieProviderService implements ProviderService<MovieOverview> {
    private static final Category CATEGORY = Category.MOVIES;

    private final FxLib fxLib;
    private final PopcornFx instance;
    private final ExecutorService executorService;

    @Override
    public boolean supports(Category category) {
        return category == CATEGORY;
    }

    @Override
    public CompletableFuture<List<MovieOverview>> getPage(Genre genre, SortBy sortBy, int page) {
        return CompletableFuture.supplyAsync(() -> getPage(genre, sortBy, "", page), executorService);
    }

    @Override
    public CompletableFuture<List<MovieOverview>> getPage(Genre genre, SortBy sortBy, int page, String keywords) {
        return CompletableFuture.supplyAsync(() -> getPage(genre, sortBy, keywords, page), executorService);
    }

    @Override
    public CompletableFuture<Media> retrieveDetails(Media media) {
        return CompletableFuture.supplyAsync(() -> getInternalDetails(media), executorService);
    }

    @Override
    public void resetApiAvailability() {
        fxLib.reset_movie_apis(instance);
    }

    public List<MovieOverview> getPage(Genre genre, SortBy sortBy, String keywords, int page) {
        try (var mediaResult = fxLib.retrieve_available_movies(instance, genre, sortBy, keywords, page)) {
            if (mediaResult.getTag() == MediaSetResult.Tag.Ok) {
                var movies = Optional.ofNullable(mediaResult.getUnion())
                        .map(MediaSetResult.MediaSetResultUnion::getOk)
                        .map(MediaSetResult.OkBody::getMediaSet)
                        .map(MediaSet::getMovies)
                        .orElse(Collections.emptyList());
                log.debug("Retrieved movies {}", movies);

                return movies;
            } else {
                var mediaError = mediaResult.getUnion().getErr().getMediaError();
                switch (mediaError) {
                    case NoAvailableProviders -> throw new MediaRetrievalException(mediaError.getMessage());
                    case NoItemsFound -> {
                        return Collections.emptyList();
                    }
                    default -> throw new MediaException(mediaError.getMessage());
                }
            }
        }
    }

    private MovieDetails getInternalDetails(Media media) {
        var result = fxLib.retrieve_media_details(instance, MediaItem.from(media));
        log.debug("Retrieved media details result {}", result);

        if (result.getTag() == MediaResult.Tag.Ok) {
            var mediaItem = result.getUnion().getOk().getMediaItem();
            return (MovieDetails) mediaItem.getMedia();
        } else {
            var error = result.getUnion().getErr();
            switch (error.getMediaError()) {
                case NoAvailableProviders -> throw new MediaRetrievalException("no providers are available");
                default -> throw new MediaException("failed to retrieve media details");
            }
        }
    }
}
