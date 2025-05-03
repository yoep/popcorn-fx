package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.lib.ipc.protobuf.WatchedEvent;
import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.media.watched.WatchedEventListener;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.OverlayItemListener;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.OverlayItemMetadataProvider;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.Parent;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.Pane;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.List;
import java.util.Objects;
import java.util.ResourceBundle;

import static java.util.Arrays.asList;

@Slf4j
public class TvMediaCardComponent extends AbstractCardComponent implements Initializable, WatchedEventListener {
    static final String WATCHED_STYLE_CLASS = "watched";

    protected final List<OverlayItemListener> listeners;
    protected final OverlayItemMetadataProvider metadataProvider;

    @FXML
    Pane posterItem;

    public TvMediaCardComponent(Media media,
                                ImageService imageService,
                                OverlayItemMetadataProvider metadataProvider,
                                OverlayItemListener... listeners) {
        super(imageService, media);
        this.metadataProvider = metadataProvider;
        this.listeners = asList(listeners);

        metadataProvider.addWatchedListener(this);
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        super.initialize(url, resourceBundle);
        initializeMetadata();
        initializeParentListener();
    }

    @Override
    public void onWatchedStateChanged(WatchedEvent.WatchedStateChanged event) {
        if (Objects.equals(event.getImdbId(), media.id())) {
            switchWatched(event.getNewState());
        }
    }

    protected void initializeMetadata() {
        metadataProvider.isWatched(media).thenAccept(this::switchWatched);
    }

    protected void onShowDetails() {
        synchronized (listeners) {
            listeners.forEach(e -> e.onClicked(media));
        }
    }

    protected void onParentChanged(Parent newValue) {
        if (newValue == null) {
            metadataProvider.removeWatchedListener(this);
        }
    }

    private void switchWatched(boolean isWatched) {
        Platform.runLater(() -> {
            if (isWatched) {
                posterItem.getStyleClass().add(WATCHED_STYLE_CLASS);
            } else {
                posterItem.getStyleClass().remove(WATCHED_STYLE_CLASS);
            }
        });
    }

    private void initializeParentListener() {
        posterItem.parentProperty().addListener((observable, oldValue, newValue) -> onParentChanged(newValue));
    }

    @FXML
    void showDetails() {
        onShowDetails();
    }

    @FXML
    void onKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onShowDetails();
        }
    }
}
