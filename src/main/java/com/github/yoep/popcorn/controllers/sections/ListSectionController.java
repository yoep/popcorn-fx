package com.github.yoep.popcorn.controllers.sections;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.controllers.components.ItemListener;
import com.github.yoep.popcorn.controllers.components.MediaCardComponent;
import com.github.yoep.popcorn.controls.InfiniteScrollPane;
import com.github.yoep.popcorn.favorites.FavoriteService;
import com.github.yoep.popcorn.media.providers.ProviderService;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.models.Category;
import com.github.yoep.popcorn.models.Genre;
import com.github.yoep.popcorn.models.SortBy;
import com.github.yoep.popcorn.watched.WatchedService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.lang3.StringUtils;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Controller;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.List;
import java.util.Objects;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

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

    private CompletableFuture currentLoadRequest;
    private Thread currentProcessingThread;

    @FXML
    private InfiniteScrollPane scrollPane;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeListeners();
    }

    @PostConstruct
    private void init() {
        activityManager.register(CategoryChangedActivity.class, this::onCategoryChange);
        activityManager.register(GenreChangeActivity.class, this::onGenreChange);
        activityManager.register(SortByChangeActivity.class, this::onSortByChange);
        activityManager.register(SearchActivity.class, this::onSearchChanged);
    }

    private void initializeListeners() {
        scrollPane.addListener((previousPage, newPage) -> loadMovies(newPage));
    }

    private void onCategoryChange(CategoryChangedActivity categoryActivity) {
        this.category = categoryActivity.getCategory();
        reset();

        // reset the genre & sort by as they might be different in the new category
        // these will be automatically filled in again as the category change also triggers a GenreChangeActivity & SortByChangeActivity
        this.genre = null;
        this.sortBy = null;
    }

    private void onGenreChange(GenreChangeActivity genreActivity) {
        this.genre = genreActivity.getGenre();
        reset();
        scrollPane.loadNewPage();
    }

    private void onSortByChange(SortByChangeActivity sortByActivity) {
        this.sortBy = sortByActivity.getSortBy();
        reset();
        scrollPane.loadNewPage();
    }

    private void onSearchChanged(SearchActivity activity) {
        String newValue = activity.getValue();

        if (Objects.equals(search, newValue))
            return;

        this.search = newValue;
        reset();
        scrollPane.loadNewPage();
    }

    private void loadMovies(final int page) {
        // wait for all filters to be known
        if (category == null || genre == null || sortBy == null)
            return;

        // cancel the current load request if present
        if (currentLoadRequest != null)
            currentLoadRequest.cancel(true);
        if (currentProcessingThread != null && currentProcessingThread.isAlive())
            currentProcessingThread.interrupt();

        log.trace("Retrieving media page {} for {} category", page, category);
        providerServices.stream()
                .filter(e -> e.supports(category))
                .findFirst()
                .ifPresentOrElse(provider -> retrieveMediaPage(provider, page),
                        () -> log.error("No provider service found for \"{}\" category", category));
    }

    private void retrieveMediaPage(ProviderService<? extends Media> provider, int page) {
        CompletableFuture<? extends List<? extends Media>> providerPage;

        if (StringUtils.isEmpty(search)) {
            providerPage = provider.getPage(genre, sortBy, page);
        } else {
            providerPage = provider.getPage(genre, sortBy, page, search);
        }

        currentLoadRequest = providerPage.thenAccept(this::processMediaPage);
    }

    private void processMediaPage(final List<? extends Media> mediaList) {
        // offload to a thread which we can cancel later on
        currentProcessingThread = new Thread(() -> {
            mediaList.forEach(media -> {
                MediaCardComponent mediaCardComponent = new MediaCardComponent(media, localeText, taskExecutor, createItemListener());
                Pane component = viewLoader.loadComponent("media-card.component.fxml", mediaCardComponent);

                mediaCardComponent.setIsFavorite(favoriteService.isFavorite(media));
                mediaCardComponent.setIsWatched(watchedService.isWatched(media));
                scrollPane.addItem(component);
            });
        });
        taskExecutor.execute(currentProcessingThread);
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
        providerServices.stream()
                .filter(e -> e.supports(category))
                .findFirst()
                .ifPresent(provider -> provider.showDetails(media));
    }

    private void reset() {
        scrollPane.reset();
    }
}
