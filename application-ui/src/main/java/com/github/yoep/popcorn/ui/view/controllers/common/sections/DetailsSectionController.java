package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowMovieDetailsEvent;
import com.github.yoep.popcorn.backend.events.ShowSerieDetailsEvent;
import com.github.yoep.popcorn.backend.events.ShowTorrentDetailsEvent;
import com.github.yoep.popcorn.ui.events.CloseDetailsEvent;
import com.github.yoep.popcorn.ui.events.CloseTorrentDetailsEvent;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.Node;
import javafx.scene.input.*;
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
    private final EventPublisher eventPublisher;
    private final ViewLoader viewLoader;
    private final TaskExecutor taskExecutor;

    private Pane movieDetailsPane;
    private Pane showDetailsPane;
    private Pane torrentDetailsPane;
    private Pane previousPane;

    @FXML
    Pane detailPane;

    //region PostConstruct

    @PostConstruct
    void init() {
        initializePanes();
        eventPublisher.register(ShowMovieDetailsEvent.class, event -> {
            switchContent(DetailsType.MOVIE_DETAILS);
            return event;
        });
        eventPublisher.register(ShowSerieDetailsEvent.class, event -> {
            switchContent(DetailsType.SHOW_DETAILS);
            return event;
        });
        eventPublisher.register(ShowTorrentDetailsEvent.class, event -> {
            switchContent(DetailsType.TORRENT_DETAILS);
            return event;
        });
        eventPublisher.register(CloseDetailsEvent.class, event -> {
            onDetailsClosed();
            return event;
        });
        eventPublisher.register(CloseTorrentDetailsEvent.class, event -> {
            onTorrentDetailsClosed();
            return event;
        });
    }

    private void initializePanes() {
        taskExecutor.execute(() -> {
            movieDetailsPane = viewLoader.load("common/components/details-movie.component.fxml");
            showDetailsPane = viewLoader.load("common/components/details-show.component.fxml");
            torrentDetailsPane = viewLoader.load("common/components/details-torrent.component.fxml");

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

    @FXML
    void onDetailsPressed(InputEvent event) {
        if (event instanceof KeyEvent keyEvent) {
            if (keyEvent.getCode() == KeyCode.ESCAPE || keyEvent.getCode() == KeyCode.BACK_SPACE) {
                event.consume();
                eventPublisher.publishEvent(new CloseDetailsEvent(this));
            }
        } else if (event instanceof MouseEvent mouseEvent) {
            if (mouseEvent.getButton() == MouseButton.BACK) {
                event.consume();
                eventPublisher.publishEvent(new CloseDetailsEvent(this));
            }
        }
    }

    private enum DetailsType {
        MOVIE_DETAILS,
        SHOW_DETAILS,
        TORRENT_DETAILS
    }
}
