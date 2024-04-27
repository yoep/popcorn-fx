package com.github.yoep.popcorn.ui.font.controls;

import com.github.yoep.popcorn.ui.font.PopcornFontRegistry;
import javafx.beans.property.DoubleProperty;
import javafx.beans.property.SimpleDoubleProperty;
import javafx.scene.control.Label;
import javafx.scene.paint.Color;
import javafx.scene.text.Font;
import javafx.scene.text.FontWeight;
import lombok.extern.slf4j.Slf4j;

import java.util.Objects;
import java.util.Optional;
import java.util.function.Consumer;

@Slf4j
abstract class AbstractIcon extends Label {
    public static final String STYLE_CLASS = "icon";

    private final DoubleProperty sizeFactorProperty = new SimpleDoubleProperty();
    private String fontFamily;
    private boolean updating;

    AbstractIcon(String filename) {
        Objects.requireNonNull(filename, "filename cannot be empty");
        init(filename);
    }

    AbstractIcon(String filename, String text) {
        super(text);
        Objects.requireNonNull(filename, "filename cannot be empty");
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

    <T> void setProperty(T property, Consumer<T> mapping) {
        Optional.ofNullable(property)
                .ifPresent(mapping);
    }

    @Override
    public String toString() {
        return getText();
    }

    private void init(String filename) {
        initializeFont(filename);
        initializeSizeFactor();
        initializeFontFamilyListener();
        initializeStyleClass();
    }

    private void initializeFont(String filename) {
        Font font = PopcornFontRegistry.getInstance().loadFont(filename);

        fontFamily = font.getFamily();
        setFont(font);
    }

    private void initializeSizeFactor() {
        sizeFactorProperty.addListener((observable, oldValue, newValue) -> {
            updating = true;
            Font oldFont = getFont();
            double fontSize = getActualSize(newValue.doubleValue(), oldFont.getSize());

            setFont(Font.font(fontFamily, FontWeight.findByName(oldFont.getStyle()), fontSize));
            updating = false;
        });
    }

    private void initializeFontFamilyListener() {
        // this listener prevents any changes to the font family
        fontProperty().addListener((observable, oldValue, newValue) -> {
            if (newValue.getFamily().equals(fontFamily) || updating)
                return;

            updating = true;
            double fontSize = getActualSize(sizeFactorProperty.get(), newValue.getSize());

            setFont(Font.font(fontFamily, FontWeight.findByName(newValue.getStyle()), fontSize));
            updating = false;
        });
    }

    private void initializeStyleClass() {
        this.getStyleClass().add(STYLE_CLASS);
    }

    private double getActualSize(double sizeFactor, double fontSize) {
        return sizeFactor <= 0 ? fontSize : sizeFactor * fontSize;
    }
}
