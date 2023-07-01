package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.ui.events.SuccessNotificationEvent;
import javafx.fxml.FXML;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.input.MouseEvent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

@Slf4j
@ViewController
@RequiredArgsConstructor
public class SettingsActionsComponent {
    static final String SUBTITLES_CLEANED_MESSAGE = "subtitles_cleaned";

    private final SubtitleService subtitleService;
    private final EventPublisher eventPublisher;
    private final LocaleText localeText;

    private void onCleanSubtitles() {
        subtitleService.cleanup();
        eventPublisher.publish(new SuccessNotificationEvent(this, localeText.get(SUBTITLES_CLEANED_MESSAGE)));
    }

    @FXML
    void onCleanSubtitlesClicked(MouseEvent event) {
        event.consume();
        onCleanSubtitles();
    }

    @FXML
    void onCleanSubtitlesPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
            onCleanSubtitles();
        }
    }

    @FXML
    void onCleanTorrentsClicked(MouseEvent event) {
        event.consume();
    }

    @FXML
    void onCleanTorrentsPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.ENTER) {
            event.consume();
        }
    }
}
