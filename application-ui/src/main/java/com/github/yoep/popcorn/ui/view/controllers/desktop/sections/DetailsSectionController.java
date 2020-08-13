package com.github.yoep.popcorn.ui.view.controllers.desktop.sections;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.ui.events.*;
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
    private Pane previousPane;

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
        activityManager.register(ShowMovieDetailsEvent.class, activity -> switchContent(DetailsType.MOVIE_DETAILS));
        activityManager.register(ShowSerieDetailsEvent.class, activity -> switchContent(DetailsType.SHOW_DETAILS));
        activityManager.register(ShowTorrentDetailsEvent.class, activity -> switchContent(DetailsType.TORRENT_DETAILS));
        activityManager.register(CloseDetailsEvent.class, activity -> onDetailsClosed());
        activityManager.register(CloseTorrentDetailsEvent.class, activity -> onTorrentDetailsClosed());
    }

    //endregion

    private void switchContent(DetailsType type) {
        log.trace("Switching details to type {}", type);
        var pane = new AtomicReference<Pane>();

        // store the current detail pane
        detailPane.getChildren().stream()
                .filter(e -> e instanceof Pane)
                .findFirst()
                .ifPresent(e -> previousPane = (Pane) e);

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

    private void onDetailsClosed() {
        previousPane = null;
    }

    private void onTorrentDetailsClosed() {
        // check if we're able to show the previous details content
        if (previousPane != null) {
            var type = (DetailsType) null;

            if (previousPane == movieDetailsPane)
                type = DetailsType.MOVIE_DETAILS;
            if (previousPane == showDetailsPane)
                type = DetailsType.SHOW_DETAILS;

            if (type != null) {
                switchContent(type);
                return;
            }
        }

        // close the details view
        activityManager.register(new CloseDetailsEvent() {
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
