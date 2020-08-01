package com.github.yoep.popcorn.ui.subtitles.controls;

import com.github.yoep.popcorn.ui.subtitles.Subtitle;
import com.github.yoep.popcorn.ui.subtitles.models.DecorationType;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleIndex;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleLine;
import com.github.yoep.popcorn.ui.subtitles.models.SubtitleText;
import javafx.application.Platform;
import javafx.beans.property.*;
import javafx.geometry.Pos;
import javafx.scene.Node;
import javafx.scene.control.Label;
import javafx.scene.layout.*;
import javafx.scene.paint.Color;
import javafx.scene.text.*;
import lombok.extern.slf4j.Slf4j;

import java.util.ArrayList;
import java.util.List;
import java.util.stream.Collectors;

@Slf4j
public class SubtitleTrack extends VBox {
    public static final String FONT_FAMILY_PROPERTY = "fontFamily";
    public static final String FONT_SIZE_PROPERTY = "fontSize";
    public static final String FONT_WEIGHT_PROPERTY = "fontWeight";
    public static final String DECORATION_PROPERTY = "decoration";
    public static final String OFFSET_PROPERTY = "offset";

    private static final String STYLE_CLASS = "subtitle-track";
    private static final String TRACK_LINE_STYLE_CLASS = "track-line";
    private static final String OUTLINE_STYLE_CLASS = "outline";
    private static final String OPAQUE_STYLE_CLASS = "opaque";
    private static final String SEE_THROUGH_STYLE_CLASS = "see-through";

    private final StringProperty fontFamily = new SimpleStringProperty(this, FONT_FAMILY_PROPERTY);
    private final IntegerProperty fontSize = new SimpleIntegerProperty(this, FONT_SIZE_PROPERTY);
    private final ObjectProperty<FontWeight> fontWeight = new SimpleObjectProperty<>(this, FONT_WEIGHT_PROPERTY, FontWeight.NORMAL);
    private final ObjectProperty<DecorationType> decoration = new SimpleObjectProperty<>(this, DECORATION_PROPERTY);
    private final DoubleProperty offset = new SimpleDoubleProperty(this, OFFSET_PROPERTY);

    private List<TrackLabel> labels;
    private Subtitle subtitle;
    private SubtitleIndex activeSubtitle;

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

    public DecorationType getDecoration() {
        return decoration.get();
    }

    public ObjectProperty<DecorationType> decorationProperty() {
        return decoration;
    }

    public void setDecoration(DecorationType decoration) {
        this.decoration.set(decoration);
    }

    public double getOffset() {
        return offset.get();
    }

    public DoubleProperty offsetProperty() {
        return offset;
    }

    public void setOffset(double offset) {
        this.offset.set(offset);
    }

    //endregion

    //region Setters

    /**
     * Set the subtitle for this subtitle track.
     *
     * @param subtitle The new subtitle for this track.
     */
    public void setSubtitle(Subtitle subtitle) {
        this.subtitle = subtitle;
    }

    //endregion

    //region Methods

    /**
     * Set the new time of the video playback.
     *
     * @param time The new time of the video.
     */
    public void onTimeChanged(long time) {
        if (subtitle == null || subtitle.isNone())
            return;

        var offset = (long) (getOffset() * 1000);

        subtitle.getIndexes().stream()
                .filter(e -> time >= (e.getStartTime() + offset) && time <= (e.getEndTime() + offset))
                .findFirst()
                .ifPresentOrElse(this::updateSubtitleTrack, this::clearSubtitleTrack);
    }

    /**
     * Clear the current subtitle track.
     */
    public void clear() {
        this.labels = null;
        this.subtitle = null;
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
        decoration.addListener((observable, oldValue, newValue) -> onDecorationChanged(newValue));
        offset.addListener((observable, oldValue, newValue) -> onOffsetChanged());
    }

    private void updateSubtitleTrack(SubtitleIndex subtitle) {
        if (activeSubtitle == subtitle)
            return;

        log.trace("Updating subtitle track to {}", subtitle);
        TrackFlags[] flags = new TrackFlags[]{
                decoration.get() == DecorationType.OUTLINE ? TrackFlags.OUTLINE : TrackFlags.NORMAL,
                decoration.get() == DecorationType.OPAQUE_BACKGROUND ? TrackFlags.OPAQUE_BACKGROUND : TrackFlags.NORMAL,
                decoration.get() == DecorationType.SEE_THROUGH_BACKGROUND ? TrackFlags.SEE_THROUGH_BACKGROUND : TrackFlags.NORMAL,
        };
        var lines = subtitle.getLines().stream()
                .map(line -> new TrackLine(flags, parseSubtitleLine(line, flags)))
                .collect(Collectors.toList());

        activeSubtitle = subtitle;

        Platform.runLater(() -> {
            this.getChildren().clear();
            this.getChildren().addAll(lines);
        });
    }

    private TrackLabel[] parseSubtitleLine(SubtitleLine line, TrackFlags[] flags) {
        return line.getTexts().stream()
                .map(text -> createLabel(text, flags))
                .toArray(TrackLabel[]::new);
    }

    private TrackLabel createLabel(SubtitleText text, TrackFlags[] flags) {
        var label = new TrackLabel(text, fontFamily.get(), fontSize.get(), fontWeight.get(), flags);

        if (labels == null)
            labels = new ArrayList<>();

        // store the label in case the style changes later on
        labels.add(label);

        return label;
    }

    private void clearSubtitleTrack() {
        if (this.getChildren().size() == 0)
            return;

        log.trace("Clearing subtitle track");
        labels = null;
        activeSubtitle = null;
        Platform.runLater(() -> this.getChildren().clear());
    }

    private void onFontChanged() {
        if (labels == null)
            return;

        // update current labels with new font
        labels.forEach(e -> e.update(fontFamily.get(), fontSize.get(), fontWeight.get()));
    }

    private void onDecorationChanged(DecorationType newValue) {
        if (labels == null)
            return;

        labels.forEach(e -> updateTrackLabelFlags(newValue, e));
    }

    private void onOffsetChanged() {

    }

    private void updateTrackLabelFlags(DecorationType newValue, TrackLabel trackLabel) {
        if (newValue == DecorationType.OUTLINE) {
            trackLabel.getFlags().addFlag(TrackFlags.OUTLINE);
        } else {
            trackLabel.getFlags().removeFlag(TrackFlags.OUTLINE);
        }
        if (newValue == DecorationType.OPAQUE_BACKGROUND) {
            trackLabel.getFlags().addFlag(TrackFlags.OPAQUE_BACKGROUND);
        } else {
            trackLabel.getFlags().removeFlag(TrackFlags.OPAQUE_BACKGROUND);
        }
        if (newValue == DecorationType.SEE_THROUGH_BACKGROUND) {
            trackLabel.getFlags().addFlag(TrackFlags.SEE_THROUGH_BACKGROUND);
        } else {
            trackLabel.getFlags().removeFlag(TrackFlags.SEE_THROUGH_BACKGROUND);
        }

        trackLabel.update();
    }

    //endregion

    private static class TrackLine extends TextFlow {
        private final TrackFlags flags = TrackFlags.NORMAL;

        public TrackLine(TrackFlags[] flags, Node... children) {
            super(children);
            this.flags.addFlags(flags);

            init();
        }

        private void init() {
            getStyleClass().add(TRACK_LINE_STYLE_CLASS);
            setTextAlignment(TextAlignment.CENTER);

            initializeFlags();
        }

        private void initializeFlags() {
            if (flags.hasFlag(TrackFlags.OUTLINE)) {
                getStyleClass().add(OUTLINE_STYLE_CLASS);
            } else {
                getStyleClass().remove(OUTLINE_STYLE_CLASS);
            }
            if (flags.hasFlag(TrackFlags.OPAQUE_BACKGROUND)) {
                getStyleClass().add(OPAQUE_STYLE_CLASS);
            } else {
                getStyleClass().remove(OPAQUE_STYLE_CLASS);
            }
            if (flags.hasFlag(TrackFlags.SEE_THROUGH_BACKGROUND)) {
                getStyleClass().add(SEE_THROUGH_STYLE_CLASS);
            } else {
                getStyleClass().remove(SEE_THROUGH_STYLE_CLASS);
            }
        }
    }

    private static class TrackLabel extends Label {
        private final TrackFlags flags;

        private String family;
        private int size;
        private FontWeight weight;

        private TrackLabel(SubtitleText line, String family, int size, FontWeight weight, TrackFlags... flags) {
            super(line.getText());
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

            update(this.family, size, fontWeight, fontPosture, border);
        }

        private void init(TrackFlags[] flags) {
            this.flags.addFlags(flags);
        }

        private void update(String family, int size, FontWeight fontWeight, FontPosture fontPosture, Border border) {
            setFont(Font.font(family, fontWeight, fontPosture, size));
            setBorder(border);
        }
    }
}
