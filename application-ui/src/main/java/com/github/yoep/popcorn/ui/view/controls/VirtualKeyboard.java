package com.github.yoep.popcorn.ui.view.controls;

import com.github.spring.boot.javafx.font.controls.IconSolid;
import javafx.beans.property.SimpleStringProperty;
import javafx.scene.Node;
import javafx.scene.control.Button;
import javafx.scene.control.Control;
import javafx.scene.control.Skin;
import javafx.scene.control.SkinBase;
import javafx.scene.input.KeyCode;
import javafx.scene.input.KeyEvent;
import javafx.scene.layout.GridPane;
import javafx.scene.layout.Pane;

import java.util.Optional;

/**
 * The virtual keyboard allows the user to input text through the usage of selecting a key
 * through the UI.
 */
public class VirtualKeyboard extends Control {
    static final String STYLE_CLASS = "virtual-keyboard";
    static final String LARGE_BUTTON_STYLE_CLASS = "large";
    static final String SPACE_STYLE_CLASS = "space";

    /**
     * The text value stored within the virtual keyboard.
     * This value is not shown within the UI.
     */
    private final SimpleStringProperty text = new SimpleStringProperty("");

    public VirtualKeyboard() {
        getStyleClass().add(STYLE_CLASS);
        setOnKeyPressed(this::onKeyPressed);
    }

    //region Properties

    public String getText() {
        return text.get();
    }

    public SimpleStringProperty textProperty() {
        return text;
    }

    public void setText(String text) {
        this.text.set(text);
    }

    //endregion

    @Override
    public void requestFocus() {
        Optional.ofNullable(getSkin())
                .map(e -> (VirtualKeyboardSkin) e)
                .map(e -> e.grid)
                .map(Pane::getChildren)
                .filter(e -> e.size() >= 3)
                .map(e -> e.get(2))
                .ifPresent(Node::requestFocus);
    }

    @Override
    protected Skin<?> createDefaultSkin() {
        return new VirtualKeyboardSkin(this);
    }

    private void onKeyPressed(KeyEvent event) {
        if (event.getCode() == KeyCode.BACK_SPACE) {
            event.consume();
            onBackspace();
        }
    }

    private void onBackspace() {
        var value = text.get();

        if (value == null || value.isEmpty())
            return;

        text.set(value.substring(0, value.length() - 1));
    }

    private class VirtualKeyboardSkin extends SkinBase<VirtualKeyboard> {
        GridPane grid;

        protected VirtualKeyboardSkin(VirtualKeyboard control) {
            super(control);
            init();
        }

        private void init() {
            grid = new GridPane();

            // add the space and backspace
            initSpecialButtons();
            initButtons();

            getChildren().add(grid);
        }

        private void initButtons() {
            var column = 0;
            var row = 1;

            for (char letter : "abcdefghijklmnopqrstuvwxyz1234567890".toCharArray()) {
                var button = new Button(String.valueOf(letter));
                button.setPrefWidth(button.getFont().getSize() * 2);
                button.setPrefWidth(button.getFont().getSize() * 2);

                addAction(button, () -> text.set(text.get() + button.getText()));

                grid.add(button, column, row);

                if (++column == 6) {
                    row++;
                    column = 0;
                }
            }
        }

        private void initSpecialButtons() {
            var spaceButton = new Button("_");
            var backButton = new Button();

            spaceButton.getStyleClass().add(SPACE_STYLE_CLASS);
            backButton.setGraphic(new IconSolid(IconSolid.BACKSPACE_UNICODE));

            spaceButton.getStyleClass().add(LARGE_BUTTON_STYLE_CLASS);
            backButton.getStyleClass().add(LARGE_BUTTON_STYLE_CLASS);
            addAction(spaceButton, () -> text.set(text.get() + " "));
            addAction(backButton, VirtualKeyboard.this::onBackspace);

            grid.add(spaceButton, 0, 0, 3, 1);
            grid.add(backButton, 3, 0, 3, 1);
        }

        private void addAction(Control node, Runnable action) {
            node.setOnMouseClicked(e -> {
                e.consume();
                action.run();
            });
            node.setOnKeyPressed(e -> {
                if (e.getCode() == KeyCode.ENTER) {
                    e.consume();
                    action.run();
                }
            });
        }
    }
}