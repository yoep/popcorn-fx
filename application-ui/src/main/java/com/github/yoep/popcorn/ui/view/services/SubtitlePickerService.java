package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.PopcornException;
import com.github.yoep.popcorn.backend.messages.SubtitleMessage;
import com.github.yoep.popcorn.backend.subtitles.SubtitleException;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.view.ViewManager;
import javafx.stage.FileChooser;
import javafx.stage.Window;
import lombok.extern.slf4j.Slf4j;

import java.util.Optional;

@Slf4j
public class SubtitlePickerService {
    private final LocaleText localeText;
    private final ViewManager viewManager;

    private final FileChooser fileChooser = new FileChooser();

    public SubtitlePickerService(LocaleText localeText, ViewManager viewManager) {
        this.localeText = localeText;
        this.viewManager = viewManager;
        init();
    }

    //region Methods

    /**
     * Pick a custom subtitle file based on the user input.
     * Make sure this method is invoked on the JavaFXThread.
     * <pre>
     * Platform.runLater(() -> {
     *     var customSubtitleFile = subtitlePickerService.pickCustomSubtitle();
     * });
     * </pre>
     *
     * @return Returns the custom subtitle file if one is selected, else {@link Optional#empty()} if the selection was cancelled by the user.
     */
    public Optional<String> pickCustomSubtitle() {
        var window = getWindow();
        var file = fileChooser.showOpenDialog(window);

        if (file != null) {
            // update the initial directory for next time
            fileChooser.setInitialDirectory(file.getParentFile());
            return Optional.of(file.getAbsolutePath());
        }

        return Optional.empty();
    }

    //endregion

    //region Functions

    private void init() {
        log.trace("Initializing subtitle picker service");
        var srtDescription = localeText.get(SubtitleMessage.SRT_DESCRIPTION);

        fileChooser.getExtensionFilters().addAll(
                new FileChooser.ExtensionFilter(srtDescription, "*.srt", "*.zip")
        );
    }

    private Window getWindow() {
        var primaryStage = viewManager.getPrimaryStage();

        if (primaryStage.isEmpty()) {
            throw new PopcornException("Unable to show subtitle picker, primary stage is unknown");
        }

        return primaryStage.get();
    }

    //endregion
}
