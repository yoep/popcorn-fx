package com.github.yoep.popcorn.view.controllers.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.ShowWatchlistActivity;
import com.github.yoep.popcorn.view.controllers.components.SimpleItemListener;
import com.github.yoep.popcorn.view.controllers.components.SimpleMediaCardComponent;
import com.github.yoep.popcorn.view.controls.InfiniteScrollItemFactory;
import com.github.yoep.popcorn.view.controls.InfiniteScrollPane;
import com.github.yoep.popcorn.media.providers.MediaException;
import com.github.yoep.popcorn.media.providers.ProviderService;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Movie;
import com.github.yoep.popcorn.media.providers.models.Show;
import com.github.yoep.popcorn.trakt.TraktException;
import com.github.yoep.popcorn.trakt.TraktService;
import com.github.yoep.popcorn.trakt.models.TraktType;
import com.github.yoep.popcorn.trakt.models.WatchListItem;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Component;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.*;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;

@Slf4j
@Component
@RequiredArgsConstructor
public class WatchlistSectionController implements Initializable {
    private final ActivityManager activityManager;
    private final ViewLoader viewLoader;
    private final LocaleText localeText;
    private final TraktService traktService;
    private final ProviderService<Movie> movieProviderService;
    private final ProviderService<Show> showProviderService;

    @FXML
    private InfiniteScrollPane<Media> scrollPane;

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeScrollPane();
    }

    //region PostConstruct

    @PostConstruct
    private void init() {
        activityManager.register(ShowWatchlistActivity.class, this::loadWatchlist);
    }

    private void loadWatchlist(ShowWatchlistActivity activity) {
        scrollPane.reset();
        scrollPane.loadNewPage();
    }

    //endregion

    //region Functions

    private void initializeScrollPane() {
        scrollPane.setLoaderFactory(() -> viewLoader.load("components/loading-card.component.fxml"));
        scrollPane.setItemFactory(new InfiniteScrollItemFactory<>() {
            @Override
            public CompletableFuture<List<Media>> loadPage(int page) {
                return loadItems(page)
                        .thenApply(Arrays::asList)
                        .exceptionally(throwable -> {
                            log.error(throwable.getMessage(), throwable);
                            return Collections.emptyList();
                        });
            }

            @Override
            public Node createCell(Media item) {
                return creatItemNode(item);
            }
        });
    }

    private CompletableFuture<Media[]> loadItems(int page) {
        if (page > 1)
            return CompletableFuture.completedFuture(new Media[0]);

        return traktService.getWatchlist()
                .thenApply(watchList -> watchList.stream()
                        // map the watchlist items to Media items
                        .map(item -> {
                            if (item.getType() == TraktType.MOVIE) {
                                return parseTraktMovie(item);
                            } else {
                                return parseTraktShow(item);
                            }
                        })
                        .filter(Objects::nonNull)
                        .toArray(Media[]::new));
    }

    private Node creatItemNode(Media item) {
        // load a new media card controller and inject it into the view
        var mediaCardComponent = new SimpleMediaCardComponent(item, localeText, createListener());

        return viewLoader.load("components/media-card-simple.component.fxml", mediaCardComponent);
    }

    private SimpleItemListener createListener() {
        return media -> {
            if (media instanceof Movie) {
                movieProviderService.showDetails(media);
            } else {
                showProviderService.showDetails(media);
            }
        };
    }

    private Media parseTraktMovie(WatchListItem item) {
        try {
            return movieProviderService.getDetails(item.getMovie().getIds().getImdb()).get();
        } catch (InterruptedException | ExecutionException ex) {
            log.error(ex.getMessage(), ex);
            throw new TraktException(ex.getMessage(), ex);
        }
    }

    private Media parseTraktShow(WatchListItem item) {
        var show = item.getShow();
        var imdbId = show.getIds().getImdb();

        if (imdbId == null) {
            log.warn("Unable to retrieve trakt show details, missing IMDB id for {}", item);
            return null;
        }

        try {
            return showProviderService.getDetails(imdbId).get();
        } catch (InterruptedException | ExecutionException ex) {
            if (ex.getCause() instanceof MediaException) {
                log.error(ex.getMessage(), ex);
                return null;
            } else {
                log.error(ex.getMessage(), ex);
                throw new TraktException(ex.getMessage(), ex);
            }
        }
    }

    //endregion
}
