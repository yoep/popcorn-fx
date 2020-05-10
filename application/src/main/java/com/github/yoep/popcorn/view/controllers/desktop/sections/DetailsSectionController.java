package com.github.yoep.popcorn.view.controllers.desktop.sections;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.ShowMovieDetailsActivity;
import com.github.yoep.popcorn.activities.ShowSerieDetailsActivity;
import com.github.yoep.popcorn.activities.ShowTorrentDetailsActivity;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.Node;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.task.TaskExecutor;

import javax.annotation.PostConstruct;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@RequiredArgsConstructor
public class DetailsSectionController {
    private final ActivityManager activityManager;
    private final ViewLoader viewLoader;
    private final TaskExecutor taskExecutor;

    private Pane movieDetailsPane;
    private Pane showDetailsPane;
    private Pane torrentDetailsPane;

    @FXML
    private Pane detailPane;

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializePanes();
        initializeListeners();
    }

    private void initializePanes() {
        taskExecutor.execute(() -> {
            movieDetailsPane = viewLoader.load("components/details-movie.component.fxml");
            showDetailsPane = viewLoader.load("components/details-show.component.fxml");
            torrentDetailsPane = viewLoader.load("components/details-torrent.component.fxml");

            anchor(movieDetailsPane);
            anchor(showDetailsPane);
            anchor(torrentDetailsPane);
        });
    }

    private void initializeListeners() {
        activityManager.register(ShowMovieDetailsActivity.class, activity -> switchContent(DetailsType.MOVIE_DETAILS));
        activityManager.register(ShowSerieDetailsActivity.class, activity -> switchContent(DetailsType.SHOW_DETAILS));
        activityManager.register(ShowTorrentDetailsActivity.class, activity -> switchContent(DetailsType.TORRENT_DETAILS));
    }

    //endregion

    private void switchContent(DetailsType type) {
        log.trace("Switching details to type {}", type);
        var pane = new AtomicReference<Pane>();

        switch (type) {
            case MOVIE_DETAILS:
                pane.set(movieDetailsPane);
                break;
            case SHOW_DETAILS:
                pane.set(showDetailsPane);
                break;
            case TORRENT_DETAILS:
                pane.set(torrentDetailsPane);
                break;
            default:
                log.error("Details type {} has not been implemented", type);
                break;
        }

        Platform.runLater(() -> {
            detailPane.getChildren().clear();
            detailPane.getChildren().add(pane.get());
        });
    }

    private void anchor(Node node) {
        AnchorPane.setTopAnchor(node, 0.0);
        AnchorPane.setRightAnchor(node, 0.0);
        AnchorPane.setBottomAnchor(node, 0.0);
        AnchorPane.setLeftAnchor(node, 0.0);
    }

    private enum DetailsType {
        MOVIE_DETAILS,
        SHOW_DETAILS,
        TORRENT_DETAILS
    }
}
