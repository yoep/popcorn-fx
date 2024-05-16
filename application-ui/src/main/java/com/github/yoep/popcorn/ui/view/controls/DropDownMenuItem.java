package com.github.yoep.popcorn.ui.view.controls;

import javafx.scene.Node;
import javafx.scene.control.MenuItem;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import lombok.EqualsAndHashCode;
import lombok.Getter;
import lombok.extern.slf4j.Slf4j;

import java.util.Optional;

@Slf4j
@EqualsAndHashCode(callSuper = false)
public class DropDownMenuItem<T> extends MenuItem {
    @Getter
    private final T item;
    @Getter
    private final Image image;
    private final DropDownButtonFactory<T> itemFactory;

    public DropDownMenuItem(T item, DropDownButtonFactory<T> itemFactory) {
        super(itemFactory.displayName(item));
        this.item = item;
        this.itemFactory = itemFactory;
        this.image = createImageFromGraphicResource();

        setId(itemFactory.getId(item));
        setGraphic(createGraphicNode());
    }

    /**
     * Get the <b>guaranteed</b> correct identifier of the item.
     * While {@link MenuItem.getId()} is <b>not</b> guaranteed to be correct as it might be changed through other
     * programmatic input, this method will always guarantee the correct identifier.
     *
     * @return Returns the unique identifier of the item.
     */
    public String getIdentifier() {
        return itemFactory.getId(item);
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
