package com.github.yoep.popcorn.controls;

import javafx.scene.layout.Pane;

import java.util.function.Supplier;

/**
 * Factory interface for creating a new loader item.
 */
public interface LoaderFactory extends Supplier<Pane> {
}
