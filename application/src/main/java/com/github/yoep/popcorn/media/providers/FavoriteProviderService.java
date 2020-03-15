package com.github.yoep.popcorn.media.providers;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.ErrorNotificationActivity;
import com.github.yoep.popcorn.media.favorites.FavoriteService;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Movie;
import com.github.yoep.popcorn.media.providers.models.Show;
import com.github.yoep.popcorn.messages.DetailsMessage;
import com.github.yoep.popcorn.view.models.Category;
import com.github.yoep.popcorn.view.models.Genre;
import com.github.yoep.popcorn.view.models.SortBy;
import lombok.extern.slf4j.Slf4j;
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
    private final List<ProviderService<?>> providers;
    private final LocaleText localeText;

    public FavoriteProviderService(RestTemplate restTemplate,
                                   ActivityManager activityManager,
                                   FavoriteService favoriteService,
                                   List<ProviderService<?>> providers,
                                   LocaleText localeText) {
        super(restTemplate, activityManager);
        this.favoriteService = favoriteService;
        this.providers = providers;
        this.localeText = localeText;
    }

    @Override
    public boolean supports(Category category) {
        return category == CATEGORY;
    }

    @Override
    public CompletableFuture<Media[]> getPage(Genre genre, SortBy sortBy, int page) {
        log.debug("Retrieving favorite provider page {}", page);
        if (page > 1)
            return CompletableFuture.completedFuture(new Media[0]);

        // retrieve all favorable items from the favoriteService
        // from the liked items, filter all Media items and cast them appropriately
        Stream<Media> mediaStream = favoriteService.getAll().stream()
                .filter(e -> e instanceof Media)
                .map(e -> (Media) e);

        if (!genre.isAllGenre()) {
            mediaStream = mediaStream.filter(e -> {
                if (genre.getKey().equals("movies")) {
                    return e instanceof Movie;
                } else {
                    return e instanceof Show;
                }
            });
        }

        //TODO: implement sort filtering
        return CompletableFuture.completedFuture(mediaStream.toArray(Media[]::new));
    }

    @Override
    public CompletableFuture<Media[]> getPage(Genre genre, SortBy sortBy, int page, String keywords) {
        log.debug("Retrieving favorite provider page {}", page);
        List<Media> mediaList = favoriteService.getAll().stream()
                .filter(e -> e instanceof Media)
                .map(e -> (Media) e)
                .collect(Collectors.toList());

        //TODO: implement filtering of favorites
        return CompletableFuture.completedFuture(mediaList.stream()
                .filter(e -> e.getTitle().toLowerCase().contains(keywords.toLowerCase()))
                .toArray(Media[]::new));
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
}
