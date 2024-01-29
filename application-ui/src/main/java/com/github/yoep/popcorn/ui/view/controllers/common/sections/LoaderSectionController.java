package com.github.yoep.popcorn.ui.view.controllers.common.sections;

import com.github.spring.boot.javafx.view.ViewLoader;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.LoadingStartedEvent;
import com.github.yoep.popcorn.ui.events.LoadUrlEvent;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.layout.Pane;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.task.TaskExecutor;

import java.net.URL;
import java.util.ResourceBundle;
import java.util.concurrent.atomic.AtomicReference;

@Slf4j
@RequiredArgsConstructor
public class LoaderSectionController implements Initializable {
    private final ViewLoader viewLoader;
    private final TaskExecutor taskExecutor;
    private final EventPublisher eventPublisher;

    private Pane torrentLoaderPane;
    private Pane urlLoaderPane;

    @FXML
    private Pane rootPane;

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializePanes();
        eventPublisher.register(LoadingStartedEvent.class, event -> {
            switchPane(Type.TORRENT_LOADER);
            return event;
        });
        eventPublisher.register(LoadUrlEvent.class, event -> {
            switchPane(Type.URL_LOADER);
            return event;
        });
    }

    private void initializePanes() {
        taskExecutor.execute(() -> torrentLoaderPane = viewLoader.load("common/components/loader-torrent.component.fxml"));
        taskExecutor.execute(() -> urlLoaderPane = viewLoader.load("common/components/loader-url.component.fxml"));
    }

    //endregion

    //region Functions

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
