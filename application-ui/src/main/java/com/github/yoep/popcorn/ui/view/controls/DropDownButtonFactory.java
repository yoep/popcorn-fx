package com.github.yoep.popcorn.ui.view.controls;

import javafx.scene.image.Image;
import org.springframework.lang.Nullable;

import javax.validation.constraints.NotNull;

/**
 * The item factory for the drop down button.
 */
public interface DropDownButtonFactory<T> {
    /**
     * Retrieve the display name of the item.
     *
     * @param item The item of the drop down button.
     * @return Returns the display name of the item.
     */
    @NotNull
    String displayName(@NotNull T item);

    /**
     * Retrieve the graphics icon of the item.
     *
     * @param item The item of the drop down button.
     * @return Returns the graphic of the item if applicable.
     */
    @Nullable
    Image graphicResource(@NotNull T item);
}
