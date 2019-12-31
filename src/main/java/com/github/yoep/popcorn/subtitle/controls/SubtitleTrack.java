package com.github.yoep.popcorn.subtitle.controls;

import com.github.yoep.popcorn.subtitle.models.Subtitle;
import com.github.yoep.popcorn.subtitle.models.SubtitleLine;
import javafx.application.Platform;
import javafx.beans.property.*;
import javafx.geometry.Pos;
import javafx.scene.control.Label;
import javafx.scene.layout.*;
import javafx.scene.paint.Color;
import javafx.scene.text.Font;
import javafx.scene.text.FontPosture;
import javafx.scene.text.FontWeight;
import lombok.extern.slf4j.Slf4j;

import java.util.List;
import java.util.stream.Collectors;

@Slf4j
public class SubtitleTrack extends VBox {
    public static final String FONT_FAMILY_PROPERTY = "fontFamily";
    public static final String FONT_SIZE_PROPERTY = "fontSize";
    public static final String FONT_WEIGHT_PROPERTY = "fontWeight";
    public static final String SUBTITLE_PROPERTY = "subtitle";
    public static final String OUTLINE_PROPERTY = "outline";

    private static final String STYLE_CLASS = "subtitle-track";
    private static final String TRACK_LINE_STYLE_CLASS = "track-line";
    private static final String OUTLINE_STYLE_CLASS = "outline";

    private final StringProperty fontFamily = new SimpleStringProperty(this, FONT_FAMILY_PROPERTY);
    private final IntegerProperty fontSize = new SimpleIntegerProperty(this, FONT_SIZE_PROPERTY);
    private final ObjectProperty<FontWeight> fontWeight = new SimpleObjectProperty<>(this, FONT_WEIGHT_PROPERTY, FontWeight.NORMAL);
    private final BooleanProperty outline = new SimpleBooleanProperty(this, OUTLINE_PROPERTY);
    private final ObjectProperty<Subtitle> subtitle = new SimpleObjectProperty<>(this, SUBTITLE_PROPERTY);

    private List<Subtitle> subtitles;
    private Subtitle activeSubtitle;

    //region Constructors

    public SubtitleTrack() {
        init();
    }

    //endregion

    //region Properties

    public String getFontFamily() {
        return fontFamily.get();
    }

    public StringProperty fontFamilyProperty() {
        return fontFamily;
    }

    public void setFontFamily(String fontFamily) {
        this.fontFamily.set(fontFamily);
    }

    public int getFontSize() {
        return fontSize.get();
    }

    public IntegerProperty fontSizeProperty() {
        return fontSize;
    }

    public void setFontSize(int fontSize) {
        this.fontSize.set(fontSize);
    }

    public FontWeight getFontWeight() {
        return fontWeight.get();
    }

    public ObjectProperty<FontWeight> fontWeightProperty() {
        return fontWeight;
    }

    public void setFontWeight(FontWeight fontWeight) {
        this.fontWeight.set(fontWeight);
    }

    public Subtitle getSubtitle() {
        return subtitle.get();
    }

    public ObjectProperty<Subtitle> subtitleProperty() {
        return subtitle;
    }

    public void setSubtitle(Subtitle subtitle) {
        this.subtitle.set(subtitle);
    }

    public boolean isOutline() {
        return outline.get();
    }

    public BooleanProperty outlineProperty() {
        return outline;
    }

    public void setOutline(boolean outline) {
        this.outline.set(outline);
    }

    //endregion

    //region Setters

    /**
     * Add the given subtitles to the subtitle track.
     *
     * @param subtitles The subtitles to add.
     */
    public void setSubtitles(List<Subtitle> subtitles) {
        this.subtitles = subtitles;
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
        fontFamily.addListener((observable, oldValue, newValue) -> onFontChanged());
        fontSize.addListener((observable, oldValue, newValue) -> onFontChanged());
        fontWeight.addListener((observable, oldValue, newValue) -> onFontChanged());
        outline.addListener((observable, oldValue, newValue) -> onOutlineChanged(newValue));
    }

    private void updateSubtitleTrack(Subtitle subtitle) {
        if (activeSubtitle == subtitle)
            return;

        log.trace("Updating subtitle track to {}", subtitle);
        TrackFlags[] flags = new TrackFlags[]{
                outline.get() ? TrackFlags.OUTLINE : TrackFlags.NORMAL
        };
        List<Label> labels = subtitle.getLines().stream()
                .map(line -> new TrackLabel(line, fontFamily.get(), fontSize.get(), fontWeight.get(), flags))
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
        // update current labels with new font
        getChildren().stream()
                .map(e -> (TrackLabel) e)
                .forEach(e -> e.update(fontFamily.get(), fontSize.get(), fontWeight.get()));
    }

    private void onOutlineChanged(Boolean newValue) {
        getChildren().stream()
                .map(e -> (TrackLabel) e)
                .forEach(e -> {
                    if (newValue) {
                        e.getFlags().addFlag(TrackFlags.OUTLINE);
                    } else {
                        e.getFlags().removeFlag(TrackFlags.OUTLINE);
                    }

                    e.update();
                });
    }

    //endregion

    private static class TrackLabel extends Label {
        private final SubtitleLine line;
        private final TrackFlags flags;

        private String family;
        private int size;
        private FontWeight weight;

        private TrackLabel(SubtitleLine line, String family, int size, FontWeight weight, TrackFlags... flags) {
            super(line.getText());
            this.line = line;
            this.flags = TrackFlags.from(line);
            this.family = family;
            this.size = size;
            this.weight = weight;

            init(flags);
            update();
        }

        public TrackFlags getFlags() {
            return flags;
        }

        void update(String family, int size, FontWeight weight) {
            this.family = family;
            this.size = size;
            this.weight = weight;

            update();
        }

        void update() {
            FontPosture fontPosture = FontPosture.REGULAR;
            FontWeight fontWeight = this.weight;
            Border border = Border.EMPTY;
            int size = this.size;

            if (flags.hasFlag(TrackFlags.ITALIC)) {
                fontPosture = FontPosture.ITALIC;
            }

            if (flags.hasFlag(TrackFlags.BOLD)) {
                fontWeight = FontWeight.BOLD;
            }

            if (flags.hasFlag(TrackFlags.UNDERLINE)) {
                BorderStroke borderStroke = new BorderStroke(Color.WHITE, BorderStrokeStyle.SOLID, CornerRadii.EMPTY, new BorderWidths(0, 0, 2, 0));
                border = new Border(borderStroke);
            }

            if (flags.hasFlag(TrackFlags.OUTLINE)) {
                getStyleClass().add(OUTLINE_STYLE_CLASS);
            } else {
                getStyleClass().remove(OUTLINE_STYLE_CLASS);
            }

            update(this.family, size, fontWeight, fontPosture, border);
        }

        private void init(TrackFlags[] flags) {
            this.flags.addFlags(flags);
            getStyleClass().add(TRACK_LINE_STYLE_CLASS);
        }

        private void update(String family, int size, FontWeight fontWeight, FontPosture fontPosture, Border border) {
            setFont(Font.font(family, fontWeight, fontPosture, size));
            setBorder(border);
        }
    }
}
