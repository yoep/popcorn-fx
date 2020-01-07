package com.github.yoep.popcorn.controllers.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.controllers.components.ItemListener;
import com.github.yoep.popcorn.controllers.components.MediaCardComponent;
import com.github.yoep.popcorn.controls.InfiniteScrollPane;
import com.github.yoep.popcorn.favorites.FavoriteService;
import com.github.yoep.popcorn.providers.ProviderService;
import com.github.yoep.popcorn.providers.models.Media;
import com.github.yoep.popcorn.messages.ListMessage;
import com.github.yoep.popcorn.models.Category;
import com.github.yoep.popcorn.models.Genre;
import com.github.yoep.popcorn.models.SortBy;
import com.github.yoep.popcorn.watched.WatchedService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.Label;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Controller;
import org.springframework.web.client.HttpStatusCodeException;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.List;
import java.util.Objects;
import java.util.ResourceBundle;
import java.util.concurrent.CancellationException;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@Controller
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

    private CompletableFuture<? extends List<? extends Media>> currentLoadRequest;
    private Thread currentProcessingThread;

    @FXML
    private InfiniteScrollPane scrollPane;
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
        scrollPane.pageProperty().addListener((observable, oldValue, newValue) -> loadMovies(newValue.intValue()));
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
        if (category != null && genre != null && sortBy != null)
            scrollPane.loadNewPage();
    }

    private void loadMovies(final int page) {
        // wait for all filters to be known
        // and the page to be bigger than 0
        if (page == 0 || category == null || genre == null || sortBy == null) {
            scrollPane.finished();
            return;
        }

        // hide the failed pane in case it might be visible from the last failure
        Platform.runLater(() -> failedPane.setVisible(false));

        // cancel the current load request if present
        if (currentLoadRequest != null)
            currentLoadRequest.cancel(true);
        if (currentProcessingThread != null)
            currentProcessingThread.interrupt();

        log.trace("Retrieving media page {} for {} category", page, category);
        providerServices.stream()
                .filter(e -> e.supports(category))
                .findFirst()
                .ifPresentOrElse(provider -> retrieveMediaPage(provider, page),
                        () -> log.error("No provider service found for \"{}\" category", category));
    }

    private void retrieveMediaPage(ProviderService<? extends Media> provider, int page) {
        if (StringUtils.isEmpty(search)) {
            currentLoadRequest = provider.getPage(genre, sortBy, page);
        } else {
            currentLoadRequest = provider.getPage(genre, sortBy, page, search);
        }

        currentLoadRequest.whenComplete((mediaList, throwable) -> {
            if (throwable == null) {
                onMediaRequestCompleted(mediaList);
            } else {
                onMediaRequestFailed(throwable);
            }
        });
    }

    private ItemListener createItemListener() {
        return new ItemListener() {
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

    private void onMediaRequestCompleted(final List<? extends Media> mediaList) {
        // offload to a thread which we can cancel later on
        currentProcessingThread = new Thread(() -> {
            for (Media media : mediaList) {
                // update the watched state of the media with the latest information
                media.setWatched(watchedService.isWatched(media));

                // load a new media card controller and inject it into the view
                MediaCardComponent mediaCardComponent = new MediaCardComponent(media, localeText, taskExecutor, createItemListener());
                Pane component = viewLoader.load("components/media-card.component.fxml", mediaCardComponent);

                // update the media favorite information
                mediaCardComponent.setIsFavorite(favoriteService.isFavorite(media));
                scrollPane.addItem(component);
            }

            scrollPane.finished();
        });
        taskExecutor.execute(currentProcessingThread);
    }

    private void onMediaRequestFailed(Throwable throwable) {
        // always finish the scroll pane update
        scrollPane.finished();

        // check if the media request was cancelled
        // if so, ignore this failure
        if (throwable instanceof CancellationException)
            return;

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
    }

    private void reset() {
        scrollPane.reset();
    }
}
