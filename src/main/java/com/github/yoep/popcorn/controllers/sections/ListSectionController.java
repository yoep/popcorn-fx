package com.github.yoep.popcorn.controllers.sections;

import com.github.spring.boot.javafx.ui.scale.ScaleAwareImpl;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.activities.*;
import com.github.yoep.popcorn.controllers.components.MediaCardComponent;
import com.github.yoep.popcorn.controls.InfiniteScrollPane;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.models.Category;
import com.github.yoep.popcorn.models.Genre;
import com.github.yoep.popcorn.models.SortBy;
import com.github.yoep.popcorn.services.ProviderService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Controller;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;

@Slf4j
@Controller
@RequiredArgsConstructor
public class ListSectionController extends ScaleAwareImpl implements Initializable {
    private final ActivityManager activityManager;
    private final List<ProviderService<? extends Media>> providerServices;
    private final ViewLoader viewLoader;
    private final TaskExecutor taskExecutor;

    private Category category;
    private Genre genre;
    private SortBy sortBy;

    @FXML
    private InfiniteScrollPane scrollPane;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeListeners();
    }

    @PostConstruct
    private void init() {
        activityManager.register(CategoryChangedActivity.class, activity -> {
            this.category = activity.getCategory();
            reset();
            scrollPane.loadNewPage();
        });
        activityManager.register(GenreChangeActivity.class, activity -> {
            this.genre = activity.getGenre();
            reset();
            scrollPane.loadNewPage();
        });
        activityManager.register(SortByChangeActivity.class, activity -> {
            this.sortBy = activity.getSortBy();
            reset();
            scrollPane.loadNewPage();
        });
    }

    private void initializeListeners() {
        scrollPane.addListener((previousPage, newPage) -> loadMovies(newPage));
    }

    private void loadMovies(final int page) {
        // wait for all filters to be known
        if (category == null || genre == null || sortBy == null)
            return;

        log.trace("Retrieving media page {} for {} category", page, category);
        providerServices.stream()
                .filter(e -> e.supports(category))
                .findFirst()
                .ifPresentOrElse(provider -> retrieveMediaPage(provider, page),
                        () -> log.error("No provider service found for \"{}\" category", category));
    }

    private void retrieveMediaPage(ProviderService<? extends Media> provider, int page) {
        provider.getPage(genre, sortBy, page)
                .thenAccept(this::processMediaPage);
    }

    private void processMediaPage(List<? extends Media> mediaList) {
        mediaList.forEach(media -> {
            MediaCardComponent mediaCardComponent = new MediaCardComponent(media, taskExecutor, this::onItemClicked);
            Pane component = viewLoader.loadComponent("media-card.component.fxml", mediaCardComponent);

            scrollPane.addItem(component);
        });
    }

    private void onItemClicked(Media media) {
        activityManager.register((DetailsShowActivity) () -> media);
    }

    private void reset() {
        scrollPane.reset();
    }
}
