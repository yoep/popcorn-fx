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
import org.springframework.context.ApplicationEventPublisher;
import org.springframework.context.event.EventListener;
import org.springframework.core.task.TaskExecutor;

import javax.annotation.PostConstruct;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@RequiredArgsConstructor
public class DetailsSectionController {
    private final ApplicationEventPublisher eventPublisher;
    private final ViewLoader viewLoader;
    private final TaskExecutor taskExecutor;

    private Pane movieDetailsPane;
    private Pane showDetailsPane;
    private Pane torrentDetailsPane;
    private Pane previousPane;

    @FXML
    private Pane detailPane;

    //region Methods

    @EventListener(ShowMovieDetailsEvent.class)
    public void onShowMovieDetails() {
        switchContent(DetailsType.MOVIE_DETAILS);
    }

    @EventListener(ShowSerieDetailsEvent.class)
    public void onShowSerieDetails() {
        switchContent(DetailsType.SHOW_DETAILS);
    }

    @EventListener(ShowTorrentDetailsEvent.class)
    public void onShowTorrentDetails() {
        switchContent(DetailsType.TORRENT_DETAILS);
    }

    @EventListener(CloseDetailsEvent.class)
    public void onCloseDetails() {
        onDetailsClosed();
    }

    @EventListener(CloseTorrentDetailsEvent.class)
    public void onCloseTorrentDetails() {
        onTorrentDetailsClosed();
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializePanes();
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
        eventPublisher.publishEvent(new CloseDetailsEvent(this));
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
