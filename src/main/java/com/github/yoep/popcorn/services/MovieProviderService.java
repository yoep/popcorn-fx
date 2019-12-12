package com.github.yoep.popcorn.services;

import com.github.yoep.popcorn.media.providers.MoviesProvider;
import com.github.yoep.popcorn.media.providers.models.Movie;
import com.github.yoep.popcorn.models.Genre;
import com.github.yoep.popcorn.models.SortBy;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Service;

import java.util.List;
import java.util.concurrent.CompletableFuture;

@Service
@RequiredArgsConstructor
public class MovieProviderService implements ProviderService<Movie> {
    private final MoviesProvider moviesProvider;

    @Override
    public CompletableFuture<List<Movie>> getPage(Genre genre, SortBy sortBy, int page) {
        return CompletableFuture.completedFuture(moviesProvider.getPage(genre, sortBy, page));
    }
}
