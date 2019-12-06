package com.github.yoep.popcorn.controllers.sections;

import com.github.spring.boot.javafx.ui.scale.ScaleAwareImpl;
import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.controllers.components.ItemComponent;
import com.github.yoep.popcorn.controls.InfiniteScrollPane;
import com.github.yoep.popcorn.services.MovieService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.geometry.Insets;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import org.springframework.stereotype.Controller;

import java.net.URL;
import java.util.ResourceBundle;

@Controller
@RequiredArgsConstructor
public class ListSectionController extends ScaleAwareImpl implements Initializable {
    private final MovieService movieService;
    private final ViewLoader viewLoader;
    private int currentPageIndex;

    @FXML
    private InfiniteScrollPane scrollPane;

    @Override
    public void initialize(URL location, ResourceBundle resources) {
        initializeListPane();
        initializeListeners();
    }

    /**
     * Reset the list view to an empty view
     */
    public void reset() {
        currentPageIndex = 0;
        Platform.runLater(() -> scrollPane.getItemsPane().getChildren().clear());
    }

    private void initializeListPane() {
        scrollPane.getItemsPane().setPadding(new Insets(0, 10, 0, 10));
        loadMovies(++currentPageIndex);
    }

    private void loadMovies(int page) {
        movieService.getPage(page)
                .thenAccept(movies -> movies.forEach(movie -> {
                    Pane component = viewLoader.loadComponent("item.component.fxml", new ItemComponent(movie));

                    Platform.runLater(() -> scrollPane.getItemsPane().getChildren().add(component));
                }));
    }

    private void initializeListeners() {
        scrollPane.addListener((previousPage, newPage) -> loadMovies(newPage));
    }
}
