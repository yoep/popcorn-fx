package com.github.yoep.popcorn.media.providers;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.ErrorNotificationActivity;
import com.github.yoep.popcorn.media.favorites.FavoriteService;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Movie;
import com.github.yoep.popcorn.media.providers.models.Show;
import com.github.yoep.popcorn.media.watched.WatchedService;
import com.github.yoep.popcorn.messages.DetailsMessage;
import com.github.yoep.popcorn.view.models.Category;
import com.github.yoep.popcorn.view.models.Genre;
import com.github.yoep.popcorn.view.models.SortBy;
import lombok.extern.slf4j.Slf4j;
import org.springframework.data.domain.Page;
import org.springframework.data.domain.PageImpl;
import org.springframework.data.domain.PageRequest;
import org.springframework.stereotype.Service;
import org.springframework.web.client.RestTemplate;

import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.stream.Collectors;
import java.util.stream.Stream;

@Slf4j
@Service
public class FavoriteProviderService extends AbstractProviderService<Media> {
    private static final Category CATEGORY = Category.FAVORITES;

    private final FavoriteService favoriteService;
    private final WatchedService watchedService;
    private final List<ProviderService<?>> providers;
    private final LocaleText localeText;

    public FavoriteProviderService(RestTemplate restTemplate,
                                   ActivityManager activityManager,
                                   FavoriteService favoriteService,
                                   WatchedService watchedService,
                                   List<ProviderService<?>> providers,
                                   LocaleText localeText) {
        super(restTemplate, activityManager);
        this.favoriteService = favoriteService;
        this.watchedService = watchedService;
        this.providers = providers;
        this.localeText = localeText;
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
        Stream<Media> mediaStream = favoriteService.getAll().stream()
                .filter(e -> e instanceof Media)
                .map(e -> (Media) e)
                .peek(e -> e.setWatched(watchedService.isWatched(e)))
                .sorted(this::sortByWatchedState);

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

        //TODO: implement sort filtering
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

        //TODO: implement filtering of favorites
        return CompletableFuture.completedFuture(new PageImpl<>(items, PageRequest.of(page, MAX_ITEMS), items.size()));
    }

    @Override
    public CompletableFuture<Media> getDetails(String imdbId) {
        throw new UnsupportedOperationException();
    }

    @Override
    public void showDetails(Media media) {
        if (media instanceof Movie) {
            providers.stream()
                    .filter(e -> e.supports(Category.MOVIES))
                    .findFirst()
                    .ifPresent(e -> showDetails(e, media));
        } else {
            providers.stream()
                    .filter(e -> e.supports(Category.SERIES))
                    .findFirst()
                    .ifPresent(e -> showDetails(e, media));
        }
    }

    private void showDetails(ProviderService<?> provider, Media media) {
        try {
            provider.showDetails(media);
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
            activityManager.register((ErrorNotificationActivity) () -> localeText.get(DetailsMessage.DETAILS_FAILED_TO_LOAD));
        }
    }

    private int sortByWatchedState(Media o1, Media o2) {
        // make sure movies are always before the shows
        if (o1 instanceof Movie && o2 instanceof Show)
            return -1;
        if (o1 instanceof Show && o2 instanceof Movie)
            return 1;

        // sort by the watched state of the media items
        if (o1.isWatched() && o2.isWatched())
            return 0;
        if (o1.isWatched())
            return 1;
        if (o2.isWatched())
            return -1;

        return 0;
    }
}
