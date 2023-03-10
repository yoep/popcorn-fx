package com.github.yoep.popcorn.ui.view.controllers.tv.components;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.ui.events.RequestSearchFocus;
import javafx.fxml.FXML;
import javafx.scene.input.MouseEvent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

@Slf4j
@RequiredArgsConstructor
public class TvSidebarSearchComponent {
    private final EventPublisher eventPublisher;

    @FXML
    void onSearchClicked(MouseEvent event) {
        event.consume();
        eventPublisher.publish(new RequestSearchFocus(this));
    }
}
