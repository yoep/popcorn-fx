package com.github.yoep.player.popcorn.controls;

import javafx.beans.property.LongProperty;
import javafx.beans.property.ReadOnlyDoubleProperty;
import javafx.beans.property.ReadOnlyDoubleWrapper;
import javafx.beans.property.SimpleLongProperty;
import javafx.scene.Node;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.StackPane;
import lombok.extern.slf4j.Slf4j;

@Slf4j
public class ProgressControl extends AnchorPane {
    public static final String DURATION_PROPERTY = "duration";
    public static final String TIME_PROPERTY = "time";
    public static final String PROGRESS_PROPERTY = "progress";

    private static final String STYLE_CLASS = "progress";
    private static final String LOAD_PROGRESS_STYLE_CLASS = "load-progress";
    private static final String PLAY_PROGRESS_STYLE_CLASS = "play-progress";
    private static final String BACKGROUND_TRACK_STYLE_CLASS = "background-track";
    private static final String TRACK_STYLE_CLASS = "track";

    private final LongProperty duration = new SimpleLongProperty(this, DURATION_PROPERTY, 0);
    private final LongProperty time = new SimpleLongProperty(this, TIME_PROPERTY, 0);
    private final ReadOnlyDoubleWrapper loadProgress = new ReadOnlyDoubleWrapper(this, PROGRESS_PROPERTY, 0);

    private final StackPane backgroundTrackPane = new StackPane();
    private final StackPane loadProgressPane = new StackPane();
    private final StackPane playProgressPane = new StackPane();

    public ProgressControl() {
        init();
    }

    //region Properties

    public long getDuration() {
        return duration.get();
    }

    public LongProperty durationProperty() {
        return duration;
    }

    public void setDuration(long duration) {
        this.duration.set(duration);
    }

    public long getTime() {
        return time.get();
    }

    public LongProperty timeProperty() {
        return time;
    }

    public void setTime(long time) {
        this.time.set(time);
    }

    /**
     * Get the current load progress value of this control.
     * The value will always be between 0 and 1 (inclusive).
     *
     * @return Returns the current progress.
     */
    public double getLoadProgress() {
        return loadProgress.get();
    }

    /**
     * Get the load progress property from this control.
     *
     * @return Returns the progress property.
     */
    public ReadOnlyDoubleProperty loadProgressProperty() {
        return loadProgress.getReadOnlyProperty();
    }

    /**
     * Set the load progress value for this control.
     * The value must be between 0 and 1 (inclusive).
     *
     * @param loadProgress The new load progress value for this control.
     * @throws IllegalArgumentException Is thrown when the progress value is invalid.
     */
    public void setLoadProgress(double loadProgress) {
        if (loadProgress < 0 || loadProgress > 1) {
            throw new IllegalArgumentException("progress must be between 0 and 1");
        }

        this.loadProgress.set(loadProgress);
    }

    //endregion

    //region Methods

    /**
     * Reset this control to it's default state.
     */
    public void reset() {
        setTime(0);
        setDuration(0);
        setLoadProgress(0);
    }

    //endregion

    //region Functions

    private void init() {
        initializeBackgroundTrack();
        initializePlayProgress();
        initializeLoadProgress();
        initializeListeners();

        this.getStyleClass().add(STYLE_CLASS);
        this.getChildren().addAll(anchor(backgroundTrackPane, false), anchor(loadProgressPane, false), anchor(playProgressPane, false));
    }

    private void initializeBackgroundTrack() {
        backgroundTrackPane.getStyleClass().addAll(BACKGROUND_TRACK_STYLE_CLASS, TRACK_STYLE_CLASS);

        this.widthProperty().addListener((observable, oldValue, newValue) -> {
            var padding = this.getPadding();
            var width = newValue.doubleValue() - padding.getLeft() - padding.getRight();

            backgroundTrackPane.setPrefWidth(width);
        });
    }

    private void initializePlayProgress() {
        playProgressPane.getStyleClass().addAll(PLAY_PROGRESS_STYLE_CLASS, TRACK_STYLE_CLASS);
        updatePlayProgress(0);
    }

    private void initializeLoadProgress() {
        loadProgressPane.getStyleClass().addAll(LOAD_PROGRESS_STYLE_CLASS, TRACK_STYLE_CLASS);
        updateLoadProgress(0);
    }

    private void initializeListeners() {
        time.addListener((observable, oldValue, newValue) -> calculatePlayProgress());
        duration.addListener((observable, oldValue, newValue) -> calculatePlayProgress());
        loadProgress.addListener((observable, oldValue, newValue) -> calculateLoadProgress());
    }

    private void calculatePlayProgress() {
        var time = getTime();
        var duration = getDuration();

        if (duration == 0)
            return;
        if (time > duration)
            time = duration;

        var width = (this.getWidth() / duration) * time;

        updatePlayProgress(width);
    }

    private void calculateLoadProgress() {
        var progress = getLoadProgress();
        var width = (this.getWidth()) * progress;

        updateLoadProgress(width);
    }

    private void updatePlayProgress(double width) {
        playProgressPane.setMinWidth(width);
        playProgressPane.setMaxWidth(width);
    }

    private void updateLoadProgress(double width) {
        loadProgressPane.setMinWidth(width);
        loadProgressPane.setMaxWidth(width);
    }

    protected <T extends Node> T anchor(T node, boolean isSliderControl) {
        var verticalAnchor = isSliderControl ? 0.0 : 2.0;

        AnchorPane.setTopAnchor(node, verticalAnchor);
        AnchorPane.setBottomAnchor(node, verticalAnchor);
        AnchorPane.setLeftAnchor(node, 0.0);

        if (isSliderControl) {
            AnchorPane.setRightAnchor(node, 0.0);
        }

        return node;
    }

    //endregion
}
