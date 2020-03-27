package com.github.yoep.popcorn.view.controllers.desktop.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.ErrorNotificationActivity;
import com.github.yoep.popcorn.activities.ShowWatchlistActivity;
import com.github.yoep.popcorn.media.providers.MediaException;
import com.github.yoep.popcorn.media.providers.ProviderService;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Movie;
import com.github.yoep.popcorn.media.providers.models.Show;
import com.github.yoep.popcorn.messages.WatchlistMessage;
import com.github.yoep.popcorn.trakt.TraktException;
import com.github.yoep.popcorn.trakt.TraktService;
import com.github.yoep.popcorn.trakt.models.TraktType;
import com.github.yoep.popcorn.trakt.models.WatchListItem;
import com.github.yoep.popcorn.view.controllers.common.SimpleItemListener;
import com.github.yoep.popcorn.view.controllers.desktop.components.SimpleMediaCardComponent;
import com.github.yoep.popcorn.view.controls.InfiniteScrollItemFactory;
import com.github.yoep.popcorn.view.controls.InfiniteScrollPane;
import com.github.yoep.popcorn.view.services.ImageService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import lombok.Builder;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.*;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;

@Slf4j
public class WatchlistSectionController implements Initializable {
    private final ActivityManager activityManager;
    private final ViewLoader viewLoader;
    private final LocaleText localeText;
    private final TraktService traktService;
    private final ProviderService<Movie> movieProviderService;
    private final ProviderService<Show> showProviderService;
    private final ImageService imageService;

    @FXML
    private InfiniteScrollPane<Media> scrollPane;

    //region Constructors

    @Builder
    public WatchlistSectionController(ActivityManager activityManager,
                                      ViewLoader viewLoader,
                                      LocaleText localeText,
                                      TraktService traktService,
                                      ProviderService<Movie> movieProviderService,
                                      ProviderService<Show> showProviderService,
                                      ImageService imageService) {
        Assert.notNull(activityManager, "activityManager cannot be null");
        Assert.notNull(viewLoader, "viewLoader cannot be null");
        Assert.notNull(localeText, "localeText cannot be null");
        Assert.notNull(traktService, "traktService cannot be null");
        Assert.notNull(movieProviderService, "movieProviderService cannot be null");
        Assert.notNull(showProviderService, "showProviderService cannot be null");
        Assert.notNull(imageService, "imageService cannot be null");
        this.activityManager = activityManager;
        this.viewLoader = viewLoader;
        this.localeText = localeText;
        this.traktService = traktService;
        this.movieProviderService = movieProviderService;
        this.showProviderService = showProviderService;
        this.imageService = imageService;
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeScrollPane();
    }

    //endregion

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
        scrollPane.setLoaderFactory(() -> viewLoader.load("common/components/loading-card.component.fxml"));
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
        var mediaCardComponent = new SimpleMediaCardComponent(item, localeText, imageService, createListener());

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
            activityManager.register((ErrorNotificationActivity) () -> localeText.get(WatchlistMessage.FAILED_TO_PARSE_MOVIE));
            return null;
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
