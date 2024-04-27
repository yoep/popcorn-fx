package com.github.yoep.popcorn.ui.view.controls;

import javafx.beans.property.LongProperty;
import javafx.beans.property.ReadOnlyDoubleProperty;
import javafx.beans.property.ReadOnlyDoubleWrapper;
import javafx.beans.property.SimpleLongProperty;
import javafx.geometry.Pos;
import javafx.scene.layout.StackPane;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;

@Slf4j
public class ProgressControl extends StackPane {
    public static final String DURATION_PROPERTY = "duration";
    public static final String TIME_PROPERTY = "time";
    public static final String PROGRESS_PROPERTY = "progress";

    static final String STYLE_CLASS = "progress";
    static final String ERROR_STYLE_CLASS = "error";
    static final String LOAD_PROGRESS_STYLE_CLASS = "load-progress";
    static final String PLAY_PROGRESS_STYLE_CLASS = "play-progress";
    static final String BACKGROUND_TRACK_STYLE_CLASS = "background-track";
    static final String TRACK_STYLE_CLASS = "track";

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
            throw new IllegalArgumentException("loadProgress must be between 0 and 1 (inclusive)");
        }

        this.loadProgress.set(loadProgress);
    }

    /**
     * Sets the error state of the element based on the given boolean value.
     *
     * @param isError true if the element should be in error state, false otherwise
     */
    public void setError(boolean isError) {
        if (isError) {
            getStyleClass().add(ERROR_STYLE_CLASS);
        } else {
            removeErrorState();
        }
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
        removeErrorState();
    }

    //endregion

    //region Functions

    private void init() {
        initializeBackgroundTrack();
        initializePlayProgress();
        initializeLoadProgress();
        initializeListeners();

        this.setAlignment(Pos.CENTER_LEFT);
        this.getStyleClass().add(STYLE_CLASS);
        this.getChildren().addAll(backgroundTrackPane, loadProgressPane, playProgressPane);
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

    private void removeErrorState() {
        getStyleClass().removeIf(e -> Objects.equals(e, ERROR_STYLE_CLASS));
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

    //endregion
}
