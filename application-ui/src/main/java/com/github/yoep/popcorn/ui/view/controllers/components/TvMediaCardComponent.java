package com.github.yoep.popcorn.ui.view.controllers.components;

import com.github.yoep.popcorn.backend.media.providers.models.Media;
import com.github.yoep.popcorn.ui.view.controllers.desktop.components.OverlayItemListener;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import lombok.extern.slf4j.Slf4j;

import java.util.List;

import static java.util.Arrays.asList;

@Slf4j
public class TvMediaCardComponent extends AbstractCardComponent implements Initializable {
    protected final List<OverlayItemListener> listeners;

    public TvMediaCardComponent(Media media, ImageService imageService,
                                OverlayItemListener... listeners) {
        super(imageService, media);
        this.listeners = asList(listeners);
    }

    protected void onShowDetails() {
        synchronized (listeners) {
            listeners.forEach(e -> e.onClicked(media));
        }
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
