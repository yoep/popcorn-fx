package com.github.yoep.popcorn.media.providers;

import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.media.favorites.FavoriteService;
import com.github.yoep.popcorn.models.Category;
import com.github.yoep.popcorn.models.Genre;
import com.github.yoep.popcorn.models.SortBy;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Movie;
import com.github.yoep.popcorn.media.providers.models.Show;
import lombok.extern.slf4j.Slf4j;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;
import org.springframework.web.client.RestTemplate;

import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.stream.Stream;

@Slf4j
@Service
public class FavoriteProviderService extends AbstractProviderService<Media> {
    private static final Category CATEGORY = Category.FAVORITES;
    private final FavoriteService favoriteService;
    //TODO: cleanup this autowired field
    @Autowired
    private List<ProviderService<?>> providers;

    public FavoriteProviderService(RestTemplate restTemplate,
                                   ActivityManager activityManager,
                                   FavoriteService favoriteService) {
        super(restTemplate, activityManager);
        this.favoriteService = favoriteService;
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

        Stream<Media> mediaStream = favoriteService.getAll().stream();

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
        List<Media> mediaList = favoriteService.getAll();

        //TODO: implement filtering of favorites
        return CompletableFuture.completedFuture(mediaList.stream()
                .filter(e -> e.getTitle().toLowerCase().contains(keywords.toLowerCase()))
                .toArray(Media[]::new));
    }

    @Override
    public void showDetails(Media media) {
        if (media instanceof Movie) {
            providers.stream()
                    .filter(e -> e.supports(Category.MOVIES))
                    .findFirst()
                    .ifPresent(e -> e.showDetails(media));
        } else {
            providers.stream()
                    .filter(e -> e.supports(Category.SERIES))
                    .findFirst()
                    .ifPresent(e -> e.showDetails(media));
        }
    }
}
