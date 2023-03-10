package com.github.yoep.popcorn.ui.view.controls;

import javafx.collections.FXCollections;
import javafx.collections.MapChangeListener;
import javafx.collections.ObservableMap;
import javafx.scene.Node;
import javafx.scene.control.Button;
import javafx.scene.control.ScrollPane;
import javafx.scene.input.KeyCode;
import javafx.scene.layout.VBox;
import lombok.Getter;
import lombok.Setter;

import java.util.List;
import java.util.Map;
import java.util.function.Consumer;

public class VerticalItemSelection<T> extends ManageableScrollPane {
    static final String STYLE_CLASS = "vertical-scroll";

    private final VBox content = new VBox();
    private final ObservableMap<T, Button> items = FXCollections.observableHashMap();

    @Getter
    @Setter
    private Consumer<T> onItemSelected;

    public VerticalItemSelection() {
        super();
        initContent();
    }

    public List<T> getItems() {
        return items.keySet().stream().toList();
    }

    public void add(T item) {
        items.put(item, createNewItem(item));
    }

    public void addAll(T... items) {
        for (T item : items) {
            add(item);
        }
    }

    public void setItems(T... items) {
        this.items.clear();
        addAll(items);
    }

    private void handleItemSelected(Node node) {
        if (onItemSelected != null) {
            for (Map.Entry<T, Button> entry : items.entrySet()) {
                if (entry.getValue() == node) {
                    onItemSelected.accept(entry.getKey());
                    return;
                }
            }
        }
    }

    private Button createNewItem(T item) {
        var button = new Button(item.toString());

        button.setOnMouseClicked(event -> {
            event.consume();
            handleItemSelected((Node) event.getSource());
        });
        button.setOnKeyPressed(event -> {
            if (event.getCode() == KeyCode.ENTER) {
                event.consume();
                handleItemSelected((Node) event.getSource());
            }
        });

        content.getChildren().add(button);
        return button;
    }

    private void initContent() {
        this.setHbarPolicy(ScrollPane.ScrollBarPolicy.NEVER);
        this.setVbarPolicy(ScrollBarPolicy.AS_NEEDED);
        this.getStyleClass().add(STYLE_CLASS);
        this.setContent(content);
        items.addListener((MapChangeListener<? super T, ? super Button>) change -> {
            if (change.wasRemoved()) {
                content.getChildren().remove(change.getValueRemoved());
            }
        });
    }
}
