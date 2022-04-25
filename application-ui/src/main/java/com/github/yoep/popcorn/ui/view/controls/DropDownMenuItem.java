package com.github.yoep.popcorn.ui.view.controls;

import javafx.scene.Node;
import javafx.scene.control.MenuItem;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import lombok.EqualsAndHashCode;
import lombok.extern.slf4j.Slf4j;

import java.util.Optional;

@Slf4j
@EqualsAndHashCode(callSuper = false)
public class DropDownMenuItem<T> extends MenuItem {
    private final T item;
    private final Image image;
    private final DropDownButtonFactory<T> itemFactory;

    public DropDownMenuItem(T item, DropDownButtonFactory<T> itemFactory) {
        super(itemFactory.displayName(item));
        this.item = item;
        this.itemFactory = itemFactory;
        this.image = createImageFromGraphicResource();

        setId(String.valueOf(item.hashCode()));
        setGraphic(createGraphicNode());
    }

    public Image getImage() {
        return image;
    }

    private Image createImageFromGraphicResource() {
        return itemFactory.graphicResource(item);
    }

    private Node createGraphicNode() {
        return Optional.ofNullable(image)
                .map(ImageView::new)
                .orElse(null);
    }
}
