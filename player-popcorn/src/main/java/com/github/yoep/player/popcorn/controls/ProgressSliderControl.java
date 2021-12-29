package com.github.yoep.player.popcorn.controls;

import javafx.beans.property.BooleanProperty;
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

    //region Functions

    private void initialize() {
        initializeSlider();

        this.getChildren().addAll(slider);
    }

    private void initializeSlider() {
        slider.valueProperty().bindBidirectional(timeProperty());
        slider.maxProperty().bind(durationProperty());
    }

    //endregion
}
