package com.github.yoep.popcorn.ui.view.controls;

import javafx.scene.layout.Pane;

import java.util.function.Supplier;

/**
 * Factory interface for creating a new loader item.
 * The loader item will be displayed while the page items are being loaded.
 */
public interface LoaderFactory extends Supplier<Pane> {
}
