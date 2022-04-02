package com.github.yoep.popcorn.ui.view.controls;

import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.beans.value.ChangeListener;
import javafx.scene.Parent;
import javafx.scene.layout.Pane;
import javafx.scene.shape.Polygon;
import org.springframework.util.Assert;

public class CornerTriangle extends Polygon {
    public static final String STYLE_CLASS = "corner";
    public static final String POSITION_PROPERTY = "position";

    private final ObjectProperty<Position> position = new SimpleObjectProperty<>(this, POSITION_PROPERTY, Position.TOP_LEFT);
    private final ChangeListener<Number> sizeListener = (observable, oldValue, newValue) -> this.onSizeChanged();

    //region Constructors

    public CornerTriangle() {
        init();
    }

    //endregion

    //region Properties

    /**
     * Get the position of the corner triangle.
     *
     * @return Returns the position of the corner triangle.
     */
    public Position getPosition() {
        return position.get();
    }

    /**
     * Get the position property of the corner triangle.
     *
     * @return Returns the position property.
     */
    public ObjectProperty<Position> positionProperty() {
        return position;
    }

    /**
     * Set the new position of the corner triangle.
     *
     * @param position The new position.
     */
    public void setPosition(Position position) {
        Assert.notNull(position, "position cannot be null");
        this.position.set(position);
    }

    //endregion

    //region Functions

    private void init() {
        initializeListeners();
        initializeStyleClass();
    }

    private void initializeListeners() {
        parentProperty().addListener((observable, oldValue, newValue) -> onParentChanged(oldValue, newValue));
        position.addListener((observable, oldValue, newValue) -> this.onPositionChanged());
    }

    private void initializeStyleClass() {
        getStyleClass().add(STYLE_CLASS);
    }

    private void onParentChanged(javafx.scene.Parent oldValue, Parent newValue) {
        if (oldValue != null) {
            var pane = (Pane) oldValue;

            pane.widthProperty().removeListener(sizeListener);
            pane.heightProperty().removeListener(sizeListener);
        }

        if (newValue != null) {
            var pane = (Pane) newValue;

            pane.widthProperty().addListener(sizeListener);
            pane.heightProperty().addListener(sizeListener);
        }
    }

    private void onSizeChanged() {
        var parent = (Pane) getParent();

        draw(parent.getWidth(), parent.getHeight());
    }

    private void onPositionChanged() {
        var parent = (Pane) getParent();

        draw(parent.getWidth(), parent.getHeight());
    }

    private void draw(double width, double height) {
        var points = getPoints();

        switch (getPosition()) {
            case TOP_LEFT -> points.setAll(
                    0.0, 0.0,
                    width, 0.0,
                    0.0, height
            );
            case TOP_RIGHT -> points.setAll(
                    0.0, 0.0,
                    width, 0.0,
                    width, height
            );
            case BOTTOM_LEFT -> points.setAll(
                    0.0, 0.0,
                    0.0, height,
                    width, height
            );
            case BOTTOM_RIGHT -> points.setAll(
                    0.0, height,
                    width, height,
                    width, 0.0
            );
        }
    }

    //endregion

    public enum Position {
        TOP_LEFT,
        TOP_RIGHT,
        BOTTOM_LEFT,
        BOTTOM_RIGHT
    }
}
