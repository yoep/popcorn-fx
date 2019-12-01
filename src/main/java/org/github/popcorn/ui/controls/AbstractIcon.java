package org.github.popcorn.ui.controls;

import javafx.beans.property.DoubleProperty;
import javafx.beans.property.SimpleDoubleProperty;
import javafx.scene.control.Label;
import javafx.scene.paint.Color;
import javafx.scene.text.Font;
import lombok.extern.log4j.Log4j2;
import org.github.popcorn.ui.FontRegistry;

import java.util.Optional;
import java.util.function.Consumer;

@Log4j2
public abstract class AbstractIcon extends Label {
    protected final DoubleProperty sizeFactorProperty = new SimpleDoubleProperty();
    protected double defaultSize;

    public AbstractIcon(String filename) {
        init(filename);
    }

    public AbstractIcon(String filename, String text) {
        super(text);
        init(filename);
    }

    public double getSizeFactor() {
        return sizeFactorProperty.get();
    }

    public void setSizeFactor(double factor) {
        sizeFactorProperty.set(factor);
    }

    public void setColor(Color color) {
        setTextFill(color);
    }

    protected <T> void setProperty(T property, Consumer<T> mapping) {
        Optional.ofNullable(property)
                .ifPresent(mapping);
    }

    private void init(String filename) {
        initializeFont(filename);
        initializeSizeFactor();
    }

    private void initializeFont(String filename) {
        Font font = FontRegistry.getInstance().loadFont(filename);
        this.defaultSize = font.getSize();

        setFont(font);
    }

    private void initializeSizeFactor() {
        sizeFactorProperty.addListener((observable, oldValue, newValue) -> setFont(new Font(getFont().getFamily(), getActualSizeFactor(newValue.doubleValue()))));
    }

    private double getActualSizeFactor(double sizeFactor) {
        return sizeFactor < 1 ? 1 : sizeFactor * defaultSize;
    }
}
