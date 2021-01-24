package com.github.yoep.popcorn.ui.media.providers;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.events.ErrorNotificationEvent;
import com.github.yoep.popcorn.ui.media.favorites.FavoriteService;
import com.github.yoep.popcorn.ui.media.favorites.FavoriteSortStrategy;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.media.providers.models.Movie;
import com.github.yoep.popcorn.ui.media.providers.models.Show;
import com.github.yoep.popcorn.ui.media.watched.WatchedService;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.view.models.Category;
import com.github.yoep.popcorn.ui.view.models.Genre;
import com.github.yoep.popcorn.ui.view.models.SortBy;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;
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
    private final LocaleText localeText;
    private final List<FavoriteSortStrategy> sortStrategies;

    public FavoriteProviderService(RestTemplate restTemplate,
                                   ApplicationEventPublisher eventPublisher,
                                   FavoriteService favoriteService,
                                   WatchedService watchedService,
                                   List<ProviderService<?>> providers,
                                   LocaleText localeText,
                                   List<FavoriteSortStrategy> sortStrategies) {
        super(restTemplate, eventPublisher);
        this.favoriteService = favoriteService;
        this.watchedService = watchedService;
        this.providers = providers;
        this.localeText = localeText;
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
                if (genre.getKey().equals("movies")) {
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
    public CompletableFuture<Boolean> showDetails(Media media) {
        Category category;

        if (media instanceof Movie) {
            category = Category.MOVIES;
        } else {
            category = Category.SERIES;
        }

        return providers.stream()
                .filter(e -> e.supports(category))
                .findFirst()
                .map(e -> showDetails(e, media))
                .orElseThrow(() -> new MediaException("Could not find ProviderService for category " + category));
    }

    private CompletableFuture<Boolean> showDetails(ProviderService<?> provider, Media media) {
        try {
            return provider.showDetails(media);
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
            eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(DetailsMessage.DETAILS_FAILED_TO_LOAD)));
        }

        return CompletableFuture.completedFuture(false);
    }

    private Media updateWatchedState(Media media) {
        var watched = watchedService.isWatched(media);

        media.setWatched(watched);

        return media;
    }
}
