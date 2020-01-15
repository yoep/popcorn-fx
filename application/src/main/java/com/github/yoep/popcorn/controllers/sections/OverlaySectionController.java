package com.github.yoep.popcorn.controllers.sections;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.ShowTorrentDetailsActivity;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Controller;

import javax.annotation.PostConstruct;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@Controller
@RequiredArgsConstructor
public class OverlaySectionController {
    private final ActivityManager activityManager;
    private final ViewLoader viewLoader;
    private final TaskExecutor taskExecutor;

    private Pane torrentDetailsPane;

    @FXML
    private Pane rootPane;

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializePanes();
        initializeListeners();
    }

    private void initializePanes() {
        taskExecutor.execute(() -> torrentDetailsPane = viewLoader.load("components/details-torrent.component.fxml"));
    }

    private void initializeListeners() {
        activityManager.register(ShowTorrentDetailsActivity.class, activity -> switchContent(Type.TORRENT_DETAILS));
    }

    //endregion

    //region Functions

    private void switchContent(Type type) {
        AtomicReference<Pane> pane = new AtomicReference<>();

        if (type == Type.TORRENT_DETAILS) {
            pane.set(torrentDetailsPane);
        }

        Platform.runLater(() -> {
            rootPane.getChildren().clear();
            rootPane.getChildren().add(pane.get());
        });
    }

    private enum Type {
        TORRENT_DETAILS
    }

    //endregion
}
