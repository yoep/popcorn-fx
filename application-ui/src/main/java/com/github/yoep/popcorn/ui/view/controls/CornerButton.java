package com.github.yoep.popcorn.ui.view.controls;

import javafx.beans.InvalidationListener;
import javafx.beans.property.ObjectProperty;
import javafx.geometry.Pos;
import javafx.scene.Node;
import javafx.scene.layout.StackPane;
import javafx.scene.paint.Paint;

public class CornerButton extends StackPane {
    public static final String STYLE_CLASS = "corner-button";

    private final CornerTriangle cornerTriangle = new CornerTriangle();

    //region Constructors

    public CornerButton() {
        init();
    }

    public CornerButton(Node... children) {
        super(children);
        init();
    }

    //endregion

    //region Properties

    /**
     * Get the position of the corner button.
     *
     * @return Returns the position of the corner button.
     */
    public CornerTriangle.Position getPosition() {
        return cornerTriangle.getPosition();
    }

    /**
     * Get the position property of the corner button.
     *
     * @return Returns the position property.
     */
    public ObjectProperty<CornerTriangle.Position> positionProperty() {
        return cornerTriangle.positionProperty();
    }

    /**
     * Set the new position of the corner button.
     *
     * @param position The new position for the corner button.
     */
    public void setPosition(CornerTriangle.Position position) {
        cornerTriangle.setPosition(position);
    }

    /**
     * Get the fill color of the corner.
     *
     * @return Returns the fill color.
     */
    public Paint getFill() {
        return cornerTriangle.getFill();
    }

    /**
     * Get the fill property of the corner.
     *
     * @return Returns the fill property.
     */
    public ObjectProperty<Paint> fillProperty() {
        return cornerTriangle.fillProperty();
    }

    /**
     * Set the new fill color of the corner.
     *
     * @param fill The new fill color of the corner.
     */
    public void setFill(Paint fill) {
        cornerTriangle.setFill(fill);
    }

    //endregion

    //region Functions

    private void init() {
        initializeCorner();
        initializeListeners();
        initializeStyleClass();
    }

    private void initializeListeners() {
        positionProperty().addListener((observable, oldValue, newValue) -> onPositionChanged(newValue));
        getChildren().addListener((InvalidationListener) observable -> onChildrenChanged());
    }

    private void initializeCorner() {
        getChildren().add(0, cornerTriangle);
        onPositionChanged(cornerTriangle.getPosition());
    }

    private void initializeStyleClass() {
        getStyleClass().add(STYLE_CLASS);
    }

    private void onChildrenChanged() {
        var position = toAlignmentPosition(getPosition());

        updatePositionsOfChildren(position);
    }

    private void onPositionChanged(CornerTriangle.Position newValue) {
        var position = toAlignmentPosition(newValue);

        updatePositionsOfChildren(position);
    }

    private void updatePositionsOfChildren(Pos position) {
        for (Node child : getChildren()) {
            StackPane.setAlignment(child, position);
        }
    }

    private Pos toAlignmentPosition(CornerTriangle.Position position) {
        return switch (position) {
            case TOP_LEFT -> Pos.TOP_LEFT;
            case TOP_RIGHT -> Pos.TOP_RIGHT;
            case BOTTOM_LEFT -> Pos.BOTTOM_LEFT;
            case BOTTOM_RIGHT -> Pos.BOTTOM_RIGHT;
        };

    }

    //endregion
}
