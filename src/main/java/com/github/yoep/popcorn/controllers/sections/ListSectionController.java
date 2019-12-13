package com.github.yoep.popcorn.controllers.sections;

import com.github.spring.boot.javafx.ui.scale.ScaleAwareImpl;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.DetailsShowActivity;
import com.github.yoep.popcorn.activities.GenreChangeActivity;
import com.github.yoep.popcorn.activities.SortByChangeActivity;
import com.github.yoep.popcorn.controllers.components.MediaCardComponent;
import com.github.yoep.popcorn.controls.InfiniteScrollPane;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Movie;
import com.github.yoep.popcorn.models.Genre;
import com.github.yoep.popcorn.models.SortBy;
import com.github.yoep.popcorn.services.ProviderService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Controller;

import javax.annotation.PostConstruct;
import java.net.URL;
import java.util.ResourceBundle;

@Controller
@RequiredArgsConstructor
public class ListSectionController extends ScaleAwareImpl implements Initializable {
    private final ActivityManager activityManager;
    private final ProviderService<Movie> movieProviderService;
    private final ViewLoader viewLoader;
    private final ContentSectionController contentController;
    private final TaskExecutor taskExecutor;
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

    private void loadMovies(int page) {
        if (genre == null || sortBy == null)
            return;

        movieProviderService.getPage(genre, sortBy, page)
                .thenAccept(movies -> movies.forEach(movie -> {
                    MediaCardComponent mediaCardComponent = new MediaCardComponent(movie, taskExecutor, this::onItemClicked);
                    Pane component = viewLoader.loadComponent("media-card.component.fxml", mediaCardComponent);

                    scrollPane.addItem(component);
                }));
    }

    private void onItemClicked(Media media) {
        activityManager.register((DetailsShowActivity) () -> media);
    }

    private void reset() {
        scrollPane.reset();
    }
}
