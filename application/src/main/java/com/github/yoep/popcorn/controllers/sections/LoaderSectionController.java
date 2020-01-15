package com.github.yoep.popcorn.controllers.sections;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.LoadTorrentActivity;
import com.github.yoep.popcorn.activities.LoadUrlActivity;
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
public class LoaderSectionController {
    private final ActivityManager activityManager;
    private final ViewLoader viewLoader;
    private final TaskExecutor taskExecutor;

    private Pane torrentLoaderPane;
    private Pane urlLoaderPane;

    @FXML
    private Pane rootPane;

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializePanes();
        initializeListeners();
    }

    private void initializePanes() {
        taskExecutor.execute(() -> torrentLoaderPane = viewLoader.load("components/loader-torrent.component.fxml"));
        taskExecutor.execute(() -> urlLoaderPane = viewLoader.load("components/loader-url.component.fxml"));
    }

    private void initializeListeners() {
        activityManager.register(LoadTorrentActivity.class, activity -> switchPane(Type.TORRENT_LOADER));
        activityManager.register(LoadUrlActivity.class, activity -> switchPane(Type.URL_LOADER));
    }

    private void switchPane(Type type) {
        AtomicReference<Pane> pane = new AtomicReference<>();

        switch (type) {
            case TORRENT_LOADER:
                pane.set(torrentLoaderPane);
                break;
            case URL_LOADER:
                pane.set(urlLoaderPane);
                break;
        }

        Platform.runLater(() -> {
            rootPane.getChildren().clear();
            rootPane.getChildren().add(pane.get());
        });
    }

    //endregion

    private enum Type {
        TORRENT_LOADER,
        URL_LOADER
    }
}