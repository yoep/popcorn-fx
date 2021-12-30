package com.github.yoep.player.popcorn.controllers.components;

import com.github.spring.boot.javafx.stereotype.ViewController;
import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.player.popcorn.listeners.PlayerSubtitleListener;
import com.github.yoep.player.popcorn.messages.MediaMessage;
import com.github.yoep.player.popcorn.services.PlayerSubtitleService;
import com.github.yoep.player.popcorn.subtitles.controls.LanguageSelection;
import com.github.yoep.popcorn.backend.subtitles.models.SubtitleInfo;
import javafx.application.Platform;
import javafx.fxml.FXML;
import javafx.fxml.Initializable;
import javafx.scene.control.ListCell;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.net.URL;
import java.util.List;
import java.util.ResourceBundle;

@Slf4j
@ViewController
@RequiredArgsConstructor
public class PlayerSubtitleComponent implements Initializable {
    private final PlayerSubtitleService subtitleService;
    private final LocaleText localeText;

    @FXML
    LanguageSelection languageSelection;

    //region Initializable

    @Override
    public void initialize(URL url, ResourceBundle resourceBundle) {
        initializeLanguageSelection();
        initializeListener();
    }

    private void initializeLanguageSelection() {
        languageSelection.getListView().setCellFactory(param -> new ListCell<>() {
            @Override
            protected void updateItem(SubtitleInfo item, boolean empty) {
                super.updateItem(item, empty);

                if (!empty) {
                    if (item.isNone()) {
                        setText(localeText.get(MediaMessage.SUBTITLE_NONE));
                    } else {
                        setText(item.getLanguage().getNativeName());
                    }
                }
            }
        });
        languageSelection.addListener(this::onSubtitleChanged);
    }

    private void initializeListener() {
        subtitleService.addListener(new PlayerSubtitleListener() {
            @Override
            public void onActiveSubtitleChanged(SubtitleInfo activeSubtitle) {
                PlayerSubtitleComponent.this.onActiveSubtitleChanged(activeSubtitle);
            }

            @Override
            public void onAvailableSubtitlesChanged(List<SubtitleInfo> subtitles, SubtitleInfo activeSubtitle) {
                PlayerSubtitleComponent.this.onAvailableSubtitlesChanged(subtitles, activeSubtitle);
            }
        });
    }

    //endregion

    //region Functions

    private void onActiveSubtitleChanged(SubtitleInfo subtitleInfo) {
        languageSelection.select(subtitleInfo);
    }

    private void onAvailableSubtitlesChanged(List<SubtitleInfo> subtitles, SubtitleInfo activeSubtitle) {
        Platform.runLater(() -> {
            languageSelection.getItems().clear();
            languageSelection.getItems().addAll(subtitles);
            languageSelection.select(activeSubtitle);
        });
    }

    private void onSubtitleChanged(SubtitleInfo subtitleInfo) {
        subtitleService.updateActiveSubtitle(subtitleInfo);
    }

    private void onSubtitleSizeChanged(int pixelChange) {
        subtitleService.updateSubtitleSizeWithSizeOffset(pixelChange);
    }

    @FXML
    void onSubtitleSmaller() {
        onSubtitleSizeChanged(-4);
    }

    @FXML
    void onSubtitleLarger() {
        onSubtitleSizeChanged(4);
    }

    //endregion
}
