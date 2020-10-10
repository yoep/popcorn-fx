package com.github.yoep.popcorn.ui.view.controls;

import javafx.application.Platform;
import javafx.event.EventHandler;
import javafx.scene.Node;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.StackPane;
import org.springframework.util.Assert;

public class Overlay extends StackPane {
    public static final String STYLE_CLASS = "overlay";

    private final EventHandler<KeyEvent> contentEventHandler = this::handleContentEvent;

    private Node originNode;
    private Node contents;

    //region Constructors

    public Overlay() {
        super();
        init();
    }

    public Overlay(Node... children) {
        super(children);
        init();
    }

    //endregion

    //region Methods

    /**
     * Show the overlay with the given contents.
     *
     * @param originNode The origin node which triggered this overlay.
     * @param contents   The contents to display in the overlay.
     */
    public void show(Node originNode, Node contents) {
        Assert.notNull(originNode, "originNode cannot be null");
        Assert.notNull(contents, "contents cannot be null");
        this.originNode = originNode;
        this.contents = contents;

        contents.addEventHandler(KeyEvent.ANY, contentEventHandler);
        getChildren().clear();
        getChildren().add(contents);

        setVisible(true);
        Platform.runLater(contents::requestFocus);
    }

    //endregion

    //region Functions

    private void init() {
        initializeStyle();
        initializeEvents();

        setVisible(false);
    }

    private void initializeStyle() {
        getStyleClass().add(STYLE_CLASS);
    }

    private void initializeEvents() {
        this.setOnKeyPressed(this::onKeyEvent);
    }

    private void handleContentEvent(KeyEvent event) {
        if (event.getEventType() != KeyEvent.KEY_RELEASED)
            onKeyEvent(event);
    }

    private void onKeyEvent(KeyEvent event) {
        var code = event.getCode();

        if (code == KeyCode.ENTER || code == KeyCode.BACK_SPACE || code == KeyCode.ESCAPE) {
            event.consume();
            onClose();
        }
    }

    private void onClose() {
        setVisible(false);

        contents.removeEventHandler(KeyEvent.ANY, contentEventHandler);
        originNode.requestFocus();
    }

    //endregion
}
