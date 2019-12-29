package com.github.yoep.popcorn.media.video.controls;

import com.github.yoep.popcorn.subtitle.models.Subtitle;
import com.github.yoep.popcorn.subtitle.models.SubtitleLine;
import javafx.application.Platform;
import javafx.beans.property.IntegerProperty;
import javafx.beans.property.SimpleIntegerProperty;
import javafx.beans.property.SimpleStringProperty;
import javafx.beans.property.StringProperty;
import javafx.geometry.Pos;
import javafx.scene.control.Label;
import javafx.scene.layout.VBox;
import javafx.scene.text.Font;
import javafx.scene.text.FontPosture;
import javafx.scene.text.FontWeight;
import lombok.Getter;
import lombok.extern.slf4j.Slf4j;

import java.util.List;
import java.util.stream.Collectors;

@Slf4j
public class SubtitleTrack extends VBox {
    private static final String STYLE_CLASS = "subtitle-track";
    private static final String TRACK_LINE_STYLE_CLASS = "track-line";

    private final StringProperty fontFamilyProperty = new SimpleStringProperty(this, "fontFamilyProperty", "Arial");
    private final IntegerProperty fontSizeProperty = new SimpleIntegerProperty(this, "fontSizeProperty", 24);
    private final StringProperty fontWeightProperty = new SimpleStringProperty(this, "fontWeightProperty", "Normal");

    private List<Subtitle> subtitles;
    private Subtitle activeSubtitle;

    // cached fonts for faster rendering
    private Font normalFont;
    private Font italicFont;
    private Font boldFont;

    //region Constructors

    public SubtitleTrack() {
        init();
    }

    //endregion

    //region Getters & Setters

    /**
     * Get the font family of the subtitle track.
     *
     * @return Returns the font family.
     */
    public String getFontFamilyProperty() {
        return fontFamilyProperty.get();
    }

    public StringProperty fontFamilyPropertyProperty() {
        return fontFamilyProperty;
    }

    public int getFontSizeProperty() {
        return fontSizeProperty.get();
    }

    public IntegerProperty fontSizePropertyProperty() {
        return fontSizeProperty;
    }

    public String getFontWeightProperty() {
        return fontWeightProperty.get();
    }

    public StringProperty fontWeightPropertyProperty() {
        return fontWeightProperty;
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
     * Set the font family of the subtitle track.
     *
     * @param family The new font family.
     */
    public void setFontFamilyProperty(String family) {
        this.fontFamilyProperty.set(family);
    }

    /**
     * Set the new font size of the subtitle track.
     *
     * @param size The new font size.
     */
    public void setFontSizeProperty(int size) {
        this.fontSizeProperty.set(size);
    }

    /**
     * Set the new font weight of the subtitle track.
     *
     * @param weight The new font weight.
     */
    public void setFontWeightProperty(String weight) {
        this.fontWeightProperty.set(weight);
    }

    //endregion

    //region Methods

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

    //endregion

    //region Functions

    private void init() {
        initializeControl();
        initializeEvents();
        onFontChanged();
    }

    private void initializeControl() {
        setFillWidth(true);
        setAlignment(Pos.CENTER);
        getStyleClass().add(STYLE_CLASS);
    }

    private void initializeEvents() {
        fontFamilyProperty.addListener((observable, oldValue, newValue) -> onFontChanged());
        fontSizeProperty.addListener((observable, oldValue, newValue) -> onFontChanged());
        fontWeightProperty.addListener((observable, oldValue, newValue) -> onFontChanged());
    }

    private void updateSubtitleTrack(Subtitle subtitle) {
        if (activeSubtitle == subtitle)
            return;

        log.trace("Updating subtitle track to {}", subtitle);
        List<Label> labels = subtitle.getLines().stream()
                .map(line -> new TrackLabel(line, TrackType.from(line), normalFont, italicFont, boldFont))
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

    private void onFontChanged() {
        var family = fontFamilyProperty.get();
        var size = fontSizeProperty.get();

        // update all cached fonts
        normalFont = Font.font(family, FontWeight.findByName(fontWeightProperty.get()), size);
        italicFont = Font.font(family, FontPosture.ITALIC, size);
        boldFont = Font.font(family, FontWeight.EXTRA_BOLD, size);

        // update current labels with new font
        getChildren().stream()
                .map(e -> (TrackLabel) e)
                .forEach(e -> e.updateFonts(normalFont, italicFont, boldFont));
    }

    //endregion

    @Getter
    private static class TrackLabel extends Label {
        private final SubtitleLine line;
        private final TrackType type;

        private Font normalFont;
        private Font italicFont;
        private Font boldFont;

        private TrackLabel(SubtitleLine line, TrackType type, Font normalFont, Font italicFont, Font boldFont) {
            super(line.getText());
            this.line = line;
            this.type = type;
            this.normalFont = normalFont;
            this.italicFont = italicFont;
            this.boldFont = boldFont;

            getStyleClass().add(TRACK_LINE_STYLE_CLASS);
            updateType();
        }

        void updateFonts(Font normalFont, Font italicFont, Font boldFont) {
            this.normalFont = normalFont;
            this.italicFont = italicFont;
            this.boldFont = boldFont;

            updateType();
        }

        private void updateType() {
            switch (type) {
                case NORMAL:
                    setFont(normalFont);
                    break;
                case ITALIC:
                    setFont(italicFont);
                    break;
                case BOLD:
                    setFont(boldFont);
                    break;
                case UNDERLINE:
                    setFont(normalFont);
                    setStyle("-fx-font-style: underline");
                    break;
            }
        }
    }

    private enum TrackType {
        NORMAL,
        ITALIC,
        BOLD,
        UNDERLINE;

        public static TrackType from(SubtitleLine line) {
            if (line.isItalic())
                return ITALIC;

            if (line.isBold())
                return BOLD;

            if (line.isUnderline())
                return UNDERLINE;

            return NORMAL;
        }
    }
}
