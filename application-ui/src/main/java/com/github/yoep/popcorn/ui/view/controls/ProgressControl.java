package com.github.yoep.popcorn.ui.view.controls;

import javafx.beans.property.LongProperty;
import javafx.beans.property.ReadOnlyDoubleProperty;
import javafx.beans.property.ReadOnlyDoubleWrapper;
import javafx.beans.property.SimpleLongProperty;
import javafx.geometry.Pos;
import javafx.scene.layout.StackPane;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

@Slf4j
public class ProgressControl extends StackPane {
    public static final String DURATION_PROPERTY = "duration";
    public static final String TIME_PROPERTY = "time";
    public static final String PROGRESS_PROPERTY = "progress";

    private static final String STYLE_CLASS = "progress";
    private static final String LOAD_PROGRESS_STYLE_CLASS = "load-progress";
    private static final String PLAY_PROGRESS_STYLE_CLASS = "play-progress";

    private final LongProperty duration = new SimpleLongProperty(this, DURATION_PROPERTY, 0);
    private final LongProperty time = new SimpleLongProperty(this, TIME_PROPERTY, 0);
    private final ReadOnlyDoubleWrapper progress = new ReadOnlyDoubleWrapper(this, PROGRESS_PROPERTY, 0);

    private final StackPane playProgress = new StackPane();
    private final StackPane loadProgress = new StackPane();

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
    public double getProgress() {
        return progress.get();
    }

    /**
     * Get the load progress property from this control.
     *
     * @return Returns the progress property.
     */
    public ReadOnlyDoubleProperty progressProperty() {
        return progress.getReadOnlyProperty();
    }

    /**
     * Set the load progress value for this control.
     * The value must be between 0 and 1 (inclusive).
     *
     * @param progress The new load progress value for this control.
     * @throws IllegalArgumentException Is thrown when the progress value is invalid.
     */
    public void setProgress(double progress) {
        Assert.isTrue(progress >= 0 && progress <= 1, "progress must be between 0 and 1");
        this.progress.set(progress);
    }

    //endregion

    //region Methods

    /**
     * Reset this control to it's default state.
     */
    public void reset() {
        setTime(0);
        setDuration(0);
        setProgress(0);
    }

    //endregion

    //region Functions

    private void init() {
        initializePlayProgress();
        initializeLoadProgress();
        initializeListeners();

        this.setAlignment(Pos.CENTER_LEFT);
        this.getStyleClass().add(STYLE_CLASS);
        this.getChildren().addAll(loadProgress, playProgress);
    }

    private void initializePlayProgress() {
        playProgress.getStyleClass().add(PLAY_PROGRESS_STYLE_CLASS);
        updatePlayProgress(0);
    }

    private void initializeLoadProgress() {
        loadProgress.getStyleClass().add(LOAD_PROGRESS_STYLE_CLASS);
        updateLoadProgress(0);
    }

    private void initializeListeners() {
        time.addListener((observable, oldValue, newValue) -> calculatePlayProgress());
        duration.addListener((observable, oldValue, newValue) -> calculatePlayProgress());
        progress.addListener((observable, oldValue, newValue) -> calculateLoadProgress());
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
        var progress = getProgress();
        var width = (this.getWidth()) * progress;

        updateLoadProgress(width);
    }

    private void updatePlayProgress(double width) {
        playProgress.setMinWidth(width);
        playProgress.setMaxWidth(width);
    }

    private void updateLoadProgress(double width) {
        loadProgress.setMinWidth(width);
        loadProgress.setMaxWidth(width);
    }

    //endregion
}
