package com.github.yoep.popcorn.providers;

import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.favorites.FavoriteService;
import com.github.yoep.popcorn.providers.models.Media;
import com.github.yoep.popcorn.providers.models.Movie;
import com.github.yoep.popcorn.providers.models.Show;
import com.github.yoep.popcorn.models.Category;
import com.github.yoep.popcorn.models.Genre;
import com.github.yoep.popcorn.models.SortBy;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Service;
import org.springframework.web.client.RestTemplate;

import java.util.Collections;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.stream.Collectors;
import java.util.stream.Stream;

@Service
public class FavoriteProviderService extends AbstractProviderService<Media> {
    private static final Category CATEGORY = Category.FAVORITES;
    private final FavoriteService favoriteService;
    //TODO: cleanup this autowired field
    @Autowired
    private List<ProviderService> providers;

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
    public CompletableFuture<List<Media>> getPage(Genre genre, SortBy sortBy, int page) {
        if (page > 1)
            return CompletableFuture.completedFuture(Collections.emptyList());

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
        return CompletableFuture.completedFuture(mediaStream.collect(Collectors.toList()));

    }

    @Override
    public CompletableFuture<List<Media>> getPage(Genre genre, SortBy sortBy, int page, String keywords) {
        List<Media> mediaList = favoriteService.getAll();

        //TODO: implement filtering of favorites
        return CompletableFuture.completedFuture(mediaList.stream()
                .filter(e -> e.getTitle().toLowerCase().contains(keywords.toLowerCase()))
                .collect(Collectors.toList()));
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
