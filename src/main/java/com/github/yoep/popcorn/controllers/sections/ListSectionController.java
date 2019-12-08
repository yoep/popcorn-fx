package com.github.yoep.popcorn.controllers.sections;

import com.github.spring.boot.javafx.ui.scale.ScaleAwareImpl;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.controllers.components.ItemComponent;
import com.github.yoep.popcorn.controls.InfiniteScrollPane;
import com.github.yoep.popcorn.providers.media.models.Movie;
import com.github.yoep.popcorn.services.ProviderService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Controller;

import java.net.URL;
import java.util.ResourceBundle;

@Controller
@RequiredArgsConstructor
public class ListSectionController extends ScaleAwareImpl implements Initializable {
    private final ProviderService<Movie> movieProviderService;
    private final ViewLoader viewLoader;
    private final ContentSectionController contentController;

    @FXML
    private InfiniteScrollPane scrollPane;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeListeners();
        initializeContent();
    }

    /**
     * Reset the list view to an empty view
     */
    public void reset() {
        scrollPane.reset();
    }

    private void initializeListeners() {
        scrollPane.addListener((previousPage, newPage) -> loadMovies(newPage));
    }

    private void initializeContent() {
        scrollPane.loadNewPage();
    }

    private void loadMovies(int page) {
        movieProviderService.getPage(page)
                .thenAccept(movies -> movies.forEach(movie -> {
                    ItemComponent itemComponent = new ItemComponent(movie, contentController::showDetails);
                    Pane component = viewLoader.loadComponent("item.component.fxml", itemComponent);

                    scrollPane.addItem(component);
                }));
    }
}
