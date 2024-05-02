package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.yoep.popcorn.backend.media.providers.Media;
import com.github.yoep.popcorn.backend.media.watched.WatchedEventCallback;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.OverlayItemListener;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.OverlayItemMetadataProvider;
import com.github.yoep.popcorn.ui.view.services.ImageService;
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
public class TvMediaCardComponent extends AbstractCardComponent implements Initializable {
    static final String WATCHED_STYLE_CLASS = "watched";

    protected final List<OverlayItemListener> listeners;
    protected final OverlayItemMetadataProvider metadataProvider;

    private final WatchedEventCallback watchedEventCallback = createWatchedCallback(media);

    @FXML
    Pane posterItem;

    public TvMediaCardComponent(Media media,
                                ImageService imageService,
                                OverlayItemMetadataProvider metadataProvider,
                                OverlayItemListener... listeners) {
        super(imageService, media);
        this.metadataProvider = metadataProvider;
        this.listeners = asList(listeners);

        metadataProvider.addListener(watchedEventCallback);
    }

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        super.initialize(url, resourceBundle);
        initializeMetadata();
        initializeParentListener();
    }

    protected void initializeMetadata() {
        switchWatched(metadataProvider.isWatched(media));
    }

    protected void onShowDetails() {
        synchronized (listeners) {
            listeners.forEach(e -> e.onClicked(media));
        }
    }

    protected void onParentChanged(Parent newValue) {
        if (newValue == null) {
            metadataProvider.removeListener(watchedEventCallback);
        }
    }

    private void switchWatched(boolean isWatched) {
        if (isWatched) {
            posterItem.getStyleClass().add(WATCHED_STYLE_CLASS);
        } else {
            posterItem.getStyleClass().remove(WATCHED_STYLE_CLASS);
        }
    }

    private void initializeParentListener() {
        posterItem.parentProperty().addListener((observable, oldValue, newValue) -> onParentChanged(newValue));
    }

    private WatchedEventCallback createWatchedCallback(Media media) {
        return event -> {
            switch (event.getTag()) {
                case WatchedStateChanged -> {
                    var stateChange = event.getUnion().getWatched_state_changed();

                    if (Objects.equals(stateChange.getImdbId(), media.getId())) {
                        switchWatched(stateChange.getNewState());
                    }
                }
            }
        };
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
