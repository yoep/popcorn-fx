package com.github.yoep.popcorn.controllers.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.controllers.components.OverlayItemListener;
import com.github.yoep.popcorn.controllers.components.OverlayMediaCardComponent;
import com.github.yoep.popcorn.controls.InfiniteScrollItemFactory;
import com.github.yoep.popcorn.controls.InfiniteScrollPane;
import com.github.yoep.popcorn.media.favorites.FavoriteService;
import com.github.yoep.popcorn.media.providers.ProviderService;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.watched.WatchedService;
import com.github.yoep.popcorn.messages.ListMessage;
import com.github.yoep.popcorn.models.Category;
import com.github.yoep.popcorn.models.Genre;
import com.github.yoep.popcorn.models.SortBy;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Node;
import javafx.scene.control.Label;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;
import org.springframework.web.client.HttpStatusCodeException;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.*;
import java.util.concurrent.CancellationException;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@Component
@RequiredArgsConstructor
public class ListSectionController implements Initializable {
    private final ActivityManager activityManager;
    private final List<ProviderService<? extends Media>> providerServices;
    private final FavoriteService favoriteService;
    private final WatchedService watchedService;
    private final ViewLoader viewLoader;
    private final TaskExecutor taskExecutor;
    private final LocaleText localeText;

    private Category category;
    private Genre genre;
    private SortBy sortBy;
    private String search;

    private CompletableFuture<? extends Media[]> currentLoadRequest;

    @FXML
    private InfiniteScrollPane<Media> scrollPane;
    @FXML
    private Pane failedPane;
    @FXML
    private Label failedText;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeScrollPane();
        initializeFailedPane();
    }

    @PostConstruct
    private void init() {
        activityManager.register(CategoryChangedActivity.class, this::onCategoryChange);
        activityManager.register(GenreChangeActivity.class, this::onGenreChange);
        activityManager.register(SortByChangeActivity.class, this::onSortByChange);
        activityManager.register(SearchActivity.class, this::onSearchChanged);
    }

    private void initializeScrollPane() {
        scrollPane.setLoaderFactory(() -> viewLoader.load("components/loading-card.component.fxml"));
        scrollPane.setItemFactory(new InfiniteScrollItemFactory<>() {
            @Override
            public CompletableFuture<List<Media>> loadPage(int page) {
                return loadItems(page)
                        .exceptionally(throwable -> onMediaRequestFailed(throwable))
                        .thenApply(Arrays::asList);
            }

            @Override
            public Node createCell(Media item) {
                return creatItemNode(item);
            }
        });
    }

    private void initializeFailedPane() {
        failedPane.setVisible(false);
    }

    private void onCategoryChange(CategoryChangedActivity categoryActivity) {
        this.category = categoryActivity.getCategory();
        // reset the genre & sort by as they might be different in the new category
        // these will be automatically filled in again as the category change also triggers a GenreChangeActivity & SortByChangeActivity
        this.genre = null;
        this.sortBy = null;

        reset();
    }

    private void onGenreChange(GenreChangeActivity genreActivity) {
        this.genre = genreActivity.getGenre();
        reset();
        invokeNewPageLoad();
    }

    private void onSortByChange(SortByChangeActivity sortByActivity) {
        this.sortBy = sortByActivity.getSortBy();
        reset();
        invokeNewPageLoad();
    }

    private void onSearchChanged(SearchActivity activity) {
        String newValue = activity.getValue();

        if (Objects.equals(search, newValue))
            return;

        this.search = newValue;
        reset();
        invokeNewPageLoad();
    }

    private void invokeNewPageLoad() {
        if (scrollPane != null && category != null && genre != null && sortBy != null)
            scrollPane.loadNewPage();
    }

    private CompletableFuture<Media[]> loadItems(final int page) {
        // wait for all filters to be known
        // and the page to be bigger than 0
        if (page == 0 || category == null || genre == null || sortBy == null) {
            return CompletableFuture.completedFuture(new Media[0]);
        }

        // hide the failed pane in case it might be visible from the last failure
        Platform.runLater(() -> failedPane.setVisible(false));

        // cancel the current load request if present
        if (currentLoadRequest != null)
            currentLoadRequest.cancel(true);

        log.trace("Retrieving media page {} for {} category", page, category);
        Optional<ProviderService<? extends Media>> provider = providerServices.stream()
                .filter(e -> e.supports(category))
                .findFirst();

        if (provider.isPresent()) {
            return retrieveMediaPage(provider.get(), page);
        } else {
            log.error("No provider service found for \"{}\" category", category);
            return CompletableFuture.completedFuture(new Media[0]);
        }
    }

    private CompletableFuture<Media[]> retrieveMediaPage(ProviderService<? extends Media> provider, int page) {
        if (StringUtils.isEmpty(search)) {
            currentLoadRequest = provider.getPage(genre, sortBy, page);
        } else {
            currentLoadRequest = provider.getPage(genre, sortBy, page, search);
        }

        return currentLoadRequest
                .thenApply(this::onMediaRequestCompleted)
                .exceptionally(this::onMediaRequestFailed);
    }

    private Node creatItemNode(Media item) {
        // update the watched & liked states of the media item with the latest information
        item.setWatched(watchedService.isWatched(item));
        item.setLiked(favoriteService.isLiked(item));

        // load a new media card controller and inject it into the view
        var mediaCardComponent = new OverlayMediaCardComponent(item, localeText, createItemListener());

        return viewLoader.load("components/media-card-overlay.component.fxml", mediaCardComponent);
    }

    private OverlayItemListener createItemListener() {
        return new OverlayItemListener() {
            @Override
            public void onClicked(Media media) {
                onItemClicked(media);
            }

            @Override
            public void onFavoriteChanged(Media media, boolean newValue) {
                if (newValue) {
                    favoriteService.addToFavorites(media);
                } else {
                    favoriteService.removeFromFavorites(media);
                }
            }

            @Override
            public void onWatchedChanged(Media media, boolean newValue) {
                if (newValue) {
                    watchedService.addToWatchList(media);
                } else {
                    watchedService.removeFromWatchList(media);
                }
            }
        };
    }

    private void onItemClicked(Media media) {
        // run on a separate thread instead of the main one
        taskExecutor.execute(() -> providerServices.stream()
                .filter(e -> e.supports(category))
                .findFirst()
                .ifPresent(provider -> provider.showDetails(media)));
    }

    private Media[] onMediaRequestCompleted(final Media[] items) {
        // filter out any duplicate items
        return Arrays.stream(items)
                .filter(e -> !scrollPane.getItems().containsKey(e))
                .toArray(Media[]::new);
    }

    private Media[] onMediaRequestFailed(Throwable throwable) {
        // check if the media request was cancelled
        // if so, ignore this failure
        if (throwable instanceof CancellationException) {
            log.trace("Media request has been cancelled by the user");
            return new Media[0];
        }

        Throwable rootCause = throwable.getCause();
        AtomicReference<String> message = new AtomicReference<>(localeText.get(ListMessage.GENERIC));
        log.error("Failed to retrieve media list, " + rootCause.getMessage(), throwable);

        if (rootCause instanceof HttpStatusCodeException) {
            HttpStatusCodeException ex = (HttpStatusCodeException) rootCause;
            message.set(localeText.get(ListMessage.API_UNAVAILABLE, ex.getStatusCode()));
        }

        Platform.runLater(() -> {
            failedText.setText(message.get());
            failedPane.setVisible(true);
        });

        return new Media[0];
    }

    private void reset() {
        scrollPane.reset();
    }
}
