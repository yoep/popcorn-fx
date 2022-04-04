package com.github.yoep.popcorn.backend.subtitles;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.spring.boot.javafx.view.ViewManager;
import com.github.yoep.popcorn.backend.messages.SubtitleMessage;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleFile;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import javafx.stage.FileChooser;
import javafx.stage.Window;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;
import java.nio.charset.Charset;
import java.util.Optional;

@Slf4j
@Service
@RequiredArgsConstructor
public class SubtitlePickerService {
    private final LocaleText localeText;
    private final ViewManager viewManager;

    private final FileChooser fileChooser = new FileChooser();

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
    public Optional<SubtitleInfo> pickCustomSubtitle() {
        var window = getWindow();
        var file = fileChooser.showOpenDialog(window);

        if (file != null) {
            var subtitleInfo = SubtitleInfo.custom();
            var subtitleFile = SubtitleFile.builder()
                    .url(file.getAbsolutePath())
                    .encoding(Charset.defaultCharset())
                    .build();

            subtitleInfo.addFile(subtitleFile);

            // update the initial directory for next time
            fileChooser.setInitialDirectory(file.getParentFile());

            return Optional.of(subtitleInfo);
        }

        return Optional.empty();
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        log.trace("Initializing subtitle picker service");
        var srtDescription = localeText.get(SubtitleMessage.SRT_DESCRIPTION);

        fileChooser.getExtensionFilters().addAll(
                new FileChooser.ExtensionFilter(srtDescription, "*.srt", "*.zip")
        );
    }

    //endregion

    //region Functions

    private Window getWindow() {
        var primaryStage = viewManager.getPrimaryStage();

        if (primaryStage.isEmpty()) {
            throw new SubtitleException("Unable to show subtitle picker, primary stage is unknown");
        }

        return primaryStage.get();
    }

    //endregion
}
