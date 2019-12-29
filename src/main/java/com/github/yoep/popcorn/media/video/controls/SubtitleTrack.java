package com.github.yoep.popcorn.media.video.controls;

import com.github.yoep.popcorn.subtitle.models.Subtitle;
import com.github.yoep.popcorn.subtitle.models.SubtitleLine;
import javafx.application.Platform;
import javafx.geometry.Pos;
import javafx.scene.control.Label;
import javafx.scene.layout.VBox;
import lombok.extern.slf4j.Slf4j;

import java.util.List;
import java.util.stream.Collectors;

@Slf4j
public class SubtitleTrack extends VBox {
    private List<Subtitle> subtitles;
    private Subtitle activeSubtitle;

    public SubtitleTrack() {
        init();
    }

    /**
     * Add the given subtitles to the subtitle track.
     *
     * @param subtitles The subtitles to add.
     */
    public void setSubtitles(List<Subtitle> subtitles) {
        this.subtitles = subtitles;
    }

    /**
     * Set the new time of the video playback.
     *
     * @param time The new time of the video.
     */
    public void onTimeChanged(long time) {
        if (subtitles == null)
            return;

        subtitles.stream()
                .filter(e -> time >= e.getStartTime() && time <= e.getEndTime())
                .findFirst()
                .ifPresentOrElse(this::updateSubtitleTrack, this::clearSubtitleTrack);
    }

    /**
     * Clear the current subtitle track.
     */
    public void clear() {
        this.subtitles = null;
        this.activeSubtitle = null;

        clearSubtitleTrack();
    }

    private void init() {
        this.setFillWidth(true);
        this.setAlignment(Pos.CENTER);
    }

    private void updateSubtitleTrack(Subtitle subtitle) {
        if (activeSubtitle == subtitle)
            return;

        log.trace("Updating subtitle track to {}", subtitle);
        SubtitleLine lastLine = subtitle.getLines().get(subtitle.getLines().size() - 1);
        List<Label> labels = subtitle.getLines().stream()
                .map(e -> {
                    String text = e.getText();

                    if (e != lastLine)
                        text += "\n";

                    Label label = new Label(text);

                    if (e.isItalic())
                        label.setStyle("-fx-font-style: italic");
                    if (e.isBold())
                        label.setStyle("-fx-font-weight: bold");
                    if (e.isUnderline())
                        label.setStyle("-fx-font-style: underline");

                    return label;
                })
                .collect(Collectors.toList());

        activeSubtitle = subtitle;

        Platform.runLater(() -> {
            this.getChildren().clear();
            this.getChildren().addAll(labels);
        });
    }

    private void clearSubtitleTrack() {
        if (this.getChildren().size() == 0)
            return;

        log.trace("Clearing subtitle track");
        activeSubtitle = null;
        Platform.runLater(() -> this.getChildren().clear());
    }
}
