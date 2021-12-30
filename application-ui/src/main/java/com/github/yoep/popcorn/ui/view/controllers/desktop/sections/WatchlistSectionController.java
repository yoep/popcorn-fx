package com.github.yoep.popcorn.ui.view.controllers.desktop.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.events.ErrorNotificationEvent;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.events.ShowSerieDetailsEvent;
import com.github.yoep.popcorn.backend.media.providers.MediaException;
import com.github.yoep.popcorn.backend.media.providers.ProviderService;
import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.backend.media.providers.models.Movie;
import com.github.yoep.popcorn.backend.media.providers.models.Show;
import com.github.yoep.popcorn.ui.events.ShowWatchlistEvent;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.messages.WatchlistMessage;
import com.github.yoep.popcorn.ui.trakt.TraktException;
import com.github.yoep.popcorn.ui.trakt.TraktService;
import com.github.yoep.popcorn.ui.trakt.models.TraktType;
import com.github.yoep.popcorn.ui.trakt.models.WatchListItem;
import com.github.yoep.popcorn.ui.view.controllers.common.SimpleItemListener;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.SimpleMediaCardComponent;
import com.github.yoep.popcorn.ui.view.controls.InfiniteScrollItemFactory;
import com.github.yoep.popcorn.ui.view.controls.InfiniteScrollPane;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.event.EventListener;

import java.net.URL;
import java.util.*;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;

@Slf4j
@RequiredArgsConstructor
public class WatchlistSectionController implements Initializable {
    private final ApplicationEventPublisher eventPublisher;
    private final ViewLoader viewLoader;
    private final LocaleText localeText;
    private final TraktService traktService;
    private final ProviderService<Movie> movieProviderService;
    private final ProviderService<Show> showProviderService;
    private final ImageService imageService;

    @FXML
    private InfiniteScrollPane<Media> scrollPane;

    //rgion Methods

    @EventListener(ShowWatchlistEvent.class)
    public void onShowWatchlist() {
        scrollPane.reset();
        scrollPane.loadNewPage();
    }

    //endregion

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeScrollPane();
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
                movieProviderService.retrieveDetails(media)
                        .whenComplete((movie, throwable) -> handleMovieDetailsResponse((Movie) movie, throwable));
            } else {
                showProviderService.retrieveDetails(media)
                        .whenComplete((show, throwable) -> handleShowDetailsResponse((Show) show, throwable));
            }
        };
    }

    private void handleMovieDetailsResponse(Movie movie, Throwable throwable) {
        if (throwable == null) {
            eventPublisher.publishEvent(new ShowMovieDetailsEvent(this, movie));
        } else {
            log.error(throwable.getMessage(), throwable);
            eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(DetailsMessage.DETAILS_FAILED_TO_LOAD)));
        }
    }

    private void handleShowDetailsResponse(Show show, Throwable throwable) {
        if (throwable == null) {
            eventPublisher.publishEvent(new ShowSerieDetailsEvent(this, show));
        } else {
            log.error(throwable.getMessage(), throwable);
            eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(DetailsMessage.DETAILS_FAILED_TO_LOAD)));
        }
    }

    private Media parseTraktMovie(WatchListItem item) {
        try {
            return movieProviderService.getDetails(item.getMovie().getIds().getImdb()).get();
        } catch (InterruptedException | ExecutionException ex) {
            log.error(ex.getMessage(), ex);
            eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(WatchlistMessage.FAILED_TO_PARSE_MOVIE)));
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
