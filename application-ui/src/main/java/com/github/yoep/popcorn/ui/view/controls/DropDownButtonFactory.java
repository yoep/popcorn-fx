package com.github.yoep.popcorn.ui.view.controls;

import javafx.scene.image.Image;


/**
 * The item factory for the drop-down button.
 */
public interface DropDownButtonFactory<T> {
    /**
     * Get the unique identifier of the item.
     *
     * @param item The item of the drop down button.
     * @return Returns the unique identifier of the item.
     */
    String getId(T item);

    /**
     * Retrieve the display name of the item.
     *
     * @param item The item of the drop down button.
     * @return Returns the display name of the item.
     */
    String displayName(T item);

    /**
     * Retrieve the graphics icon of the item.
     *
     * @param item The item of the drop down button.
     * @return Returns the graphic of the item if applicable, or null.
     */
    Image graphicResource(T item);
}
