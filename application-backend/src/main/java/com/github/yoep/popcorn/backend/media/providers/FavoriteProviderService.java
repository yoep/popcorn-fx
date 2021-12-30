package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.media.favorites.FavoriteService;
import com.github.yoep.popcorn.backend.media.favorites.FavoriteSortStrategy;
import com.github.yoep.popcorn.backend.media.filters.models.Category;
import com.github.yoep.popcorn.backend.media.filters.models.Genre;
import com.github.yoep.popcorn.backend.media.filters.models.SortBy;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.Show;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import lombok.extern.slf4j.Slf4j;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.PageImpl;
import org.springframework.data.domain.PageRequest;
import org.springframework.stereotype.Service;
import org.springframework.web.client.RestTemplate;

import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.stream.Collectors;

@Slf4j
@Service
public class FavoriteProviderService extends AbstractProviderService<Media> {
    private static final Category CATEGORY = Category.FAVORITES;

    private final FavoriteService favoriteService;
    private final WatchedService watchedService;
    private final List<ProviderService<?>> providers;
    private final List<FavoriteSortStrategy> sortStrategies;

    public FavoriteProviderService(RestTemplate restTemplate,
                                   FavoriteService favoriteService,
                                   WatchedService watchedService,
                                   List<ProviderService<?>> providers,
                                   List<FavoriteSortStrategy> sortStrategies) {
        super(restTemplate);
        this.favoriteService = favoriteService;
        this.watchedService = watchedService;
        this.providers = providers;
        this.sortStrategies = sortStrategies;
    }

    @Override
    public boolean supports(Category category) {
        return category == CATEGORY;
    }

    @Override
    public CompletableFuture<Page<Media>> getPage(Genre genre, SortBy sortBy, int page) {
        log.debug("Retrieving favorite provider page {}", page);
        if (page > 1)
            return CompletableFuture.completedFuture(Page.empty());

        // retrieve all favorable items from the favoriteService
        // from the liked items, filter all Media items and cast them appropriately
        var mediaStream = favoriteService.getAll().stream()
                .filter(e -> e instanceof Media)
                .map(e -> (Media) e)
                .map(this::updateWatchedState);

        // sort the favorites
        var sortStrategy = sortStrategies.stream()
                .filter(e -> e.support(sortBy))
                .findFirst();

        if (sortStrategy.isPresent()) {
            mediaStream = sortStrategy.get().sort(mediaStream);
        }

        if (!genre.isAllGenre()) {
            mediaStream = mediaStream.filter(e -> {
                if (genre.getKey().equals(Genre.MOVIES_KEYWORD)) {
                    return e instanceof Movie;
                } else {
                    return e instanceof Show;
                }
            });
        }

        var items = mediaStream.collect(Collectors.toList());

        return CompletableFuture.completedFuture(new PageImpl<>(items, PageRequest.of(page, MAX_ITEMS), items.size()));
    }

    @Override
    public CompletableFuture<Page<Media>> getPage(Genre genre, SortBy sortBy, int page, String keywords) {
        log.debug("Retrieving favorite provider page {}", page);
        List<Media> mediaList = favoriteService.getAll().stream()
                .filter(e -> e instanceof Media)
                .map(e -> (Media) e)
                .collect(Collectors.toList());

        var items = mediaList.stream()
                .filter(e -> e.getTitle().toLowerCase().contains(keywords.toLowerCase()))
                .collect(Collectors.toList());

        return CompletableFuture.completedFuture(new PageImpl<>(items, PageRequest.of(page, MAX_ITEMS), items.size()));
    }

    @Override
    public CompletableFuture<Media> getDetails(String imdbId) {
        throw new UnsupportedOperationException();
    }

    @Override
    public CompletableFuture<Media> retrieveDetails(Media media) {
        Category category;

        if (media instanceof Movie) {
            category = Category.MOVIES;
        } else {
            category = Category.SERIES;
        }

        return providers.stream()
                .filter(e -> e.supports(category))
                .findFirst()
                .map(e -> retrieveDetails(e, media))
                .orElseThrow(() -> new MediaException("Could not find ProviderService for category " + category));
    }

    private CompletableFuture<Media> retrieveDetails(ProviderService<?> provider, Media media) {
        try {
            return provider.retrieveDetails(media);
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
            throw new MediaDetailsException(media, "Failed to retrieve show details", ex);
        }
    }

    private Media updateWatchedState(Media media) {
        var watched = watchedService.isWatched(media);

        media.setWatched(watched);

        return media;
    }
}
