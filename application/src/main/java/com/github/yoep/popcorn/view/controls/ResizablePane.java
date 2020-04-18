package com.github.yoep.popcorn.view.controls;

import javafx.beans.property.DoubleProperty;
import javafx.beans.property.SimpleDoubleProperty;
import javafx.scene.Cursor;
import javafx.scene.Node;
import javafx.scene.Scene;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.AnchorPane;
import javafx.stage.Stage;

public class ResizablePane extends AnchorPane {
    public static final String HEADER_PROPERTY = "header";
    public static final String RESIZE_BORDER_PROPERTY = "resizeBorder";

    private final DoubleProperty header = new SimpleDoubleProperty(this, HEADER_PROPERTY, 0);
    private final DoubleProperty resizeBorder = new SimpleDoubleProperty(this, RESIZE_BORDER_PROPERTY, 4);

    private double xOffset;
    private double yOffset;
    private double xStart;
    private double yStart;
    private boolean windowDrag;
    private boolean windowResize;
    private Cursor cursor = Cursor.DEFAULT;

    //region Constructors

    public ResizablePane() {
        init();
    }

    public ResizablePane(Node... children) {
        super(children);
        init();
    }

    //endregion

    //region Properties

    /**
     * Get the header height of the pane which allows the stage to be dragged around.
     *
     * @return Returns the header height of this pane.
     */
    public double getHeader() {
        return header.get();
    }

    /**
     * Get the header height property.
     *
     * @return Return the property of the header height.
     */
    public DoubleProperty headerProperty() {
        return header;
    }

    /**
     * Set the header height of the pane which allows the stage to be dragged around.
     *
     * @param header The height of the header.
     */
    public void setHeader(double header) {
        this.header.set(header);
    }

    public double getResizeBorder() {
        return resizeBorder.get();
    }

    public DoubleProperty resizeBorderProperty() {
        return resizeBorder;
    }

    public void setResizeBorder(double resizeBorder) {
        this.resizeBorder.set(resizeBorder);
    }

    //endregion

    //region Functions

    private void init() {
        initializeEvents();
        initializeListeners();
    }

    private void initializeEvents() {
        this.setOnMouseMoved(this::onMouseMoved);
        this.setOnMousePressed(this::onMousePressed);
        this.setOnMouseDragged(this::onMouseDragged);
    }

    private void initializeListeners() {
        this.sceneProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue != null)
                onSceneChanged(newValue);
        });
    }

    private void onSceneChanged(Scene scene) {
        scene.addEventHandler(MouseEvent.MOUSE_MOVED, this::onMouseMoved);
        scene.addEventHandler(MouseEvent.MOUSE_PRESSED, this::onMousePressed);
        scene.addEventHandler(MouseEvent.MOUSE_DRAGGED, this::onMouseDragged);
    }

    private void onMouseMoved(MouseEvent event) {
        var x = event.getSceneX();
        var y = event.getSceneY();
        var width = getStage().getWidth();
        var height = getStage().getHeight();
        var border = getResizeBorder();

        if (x < border && y < border) {
            cursor = Cursor.NW_RESIZE;
        } else if (x > width - border && y < border) {
            cursor = Cursor.NE_RESIZE;
        } else if (x > width - border && y > height - border) {
            cursor = Cursor.SE_RESIZE;
        } else if (x < border && y > height - border) {
            cursor = Cursor.SW_RESIZE;
        } else if (y < border) {
            cursor = Cursor.N_RESIZE;
        } else if (x > width - border) {
            cursor = Cursor.E_RESIZE;
        } else if (y > height - border) {
            cursor = Cursor.S_RESIZE;
        } else if (x < border) {
            cursor = Cursor.W_RESIZE;
        } else {
            cursor = Cursor.DEFAULT;
        }

        windowResize = cursor != Cursor.DEFAULT;
        this.setCursor(cursor);
    }

    private void onMousePressed(MouseEvent event) {
        event.consume();

        var stage = getStage();

        xOffset = stage.getX() - event.getScreenX();
        yOffset = stage.getY() - event.getScreenY();
        xStart = stage.getWidth() - event.getSceneX();
        yStart = stage.getHeight() - event.getSceneY();
        windowDrag = isValidWindowDragEvent(event);
    }

    private void onMouseDragged(MouseEvent event) {
        if (windowDrag) {
            onWindowDrag(event);
        }
        if (windowResize) {
            onWindowResize(event);
        }
    }

    private void onWindowDrag(MouseEvent event) {
        event.consume();

        var stage = getStage();

        stage.setX(event.getScreenX() + xOffset);
        stage.setY(event.getScreenY() + yOffset);
    }

    private void onWindowResize(MouseEvent event) {
        event.consume();

        var stage = getStage();

        if (cursor == Cursor.E_RESIZE || cursor == Cursor.NE_RESIZE || cursor == Cursor.SE_RESIZE) {
            var newWidth = event.getSceneX() + xStart;

            if (newWidth >= getMinWidth())
                stage.setWidth(newWidth);
        }

        if (cursor == Cursor.S_RESIZE || cursor == Cursor.SE_RESIZE || cursor == Cursor.SW_RESIZE) {
            var newHeight = event.getSceneY() + yStart;

            if (newHeight >= getMinHeight())
                stage.setHeight(newHeight);
        }

        if (cursor == Cursor.W_RESIZE || cursor == Cursor.SW_RESIZE || cursor == Cursor.NW_RESIZE) {
            var width = stage.getX() - event.getScreenX() + stage.getWidth();

            if (width >= getMinWidth()) {
                stage.setX(event.getScreenX());
                stage.setWidth(width);
            }
        }

        if (cursor == Cursor.N_RESIZE || cursor == Cursor.NW_RESIZE || cursor == Cursor.NE_RESIZE) {
            var height = stage.getY() - event.getScreenY() + stage.getHeight();

            if (height >= getMinHeight()) {
                stage.setY(event.getScreenY());
                stage.setHeight(height);
            }
        }
    }

    private boolean isValidWindowDragEvent(MouseEvent event) {
        // check if the mouse event is a valid window drag event
        // the event should be within the header height and not a window resize event
        return !windowResize && event.getSceneY() <= getHeader();
    }

    private Stage getStage() {
        return (Stage) this.getScene().getWindow();
    }

    //endregion
}
