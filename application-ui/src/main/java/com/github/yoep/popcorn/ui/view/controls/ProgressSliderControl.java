package com.github.yoep.popcorn.ui.view.controls;

import javafx.beans.property.BooleanProperty;
import javafx.beans.property.DoubleProperty;
import javafx.scene.control.Slider;
import lombok.extern.slf4j.Slf4j;

@Slf4j
public class ProgressSliderControl extends ProgressControl {
    private final Slider slider = new Slider();

    public ProgressSliderControl() {
        super();
        initialize();
    }

    //region Properties

    public double getValue() {
        return slider.getValue();
    }

    public DoubleProperty valueProperty() {
        return slider.valueProperty();
    }

    public void setValue(double value) {
        slider.setValue(value);
    }

    public boolean isValueChanging() {
        return slider.isValueChanging();
    }

    public BooleanProperty valueChangingProperty() {
        return slider.valueChangingProperty();
    }

    public void setValueChanging(boolean value) {
        slider.setValueChanging(value);
    }

    //endregion

    //region Methods

    public void setMax(double value) {
        slider.setMax(value);
    }

    //endregion

    //region Functions

    private void initialize() {
        this.getChildren().addAll(slider);
    }

    //endregion
}
