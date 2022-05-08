package com.github.yoep.player.popcorn.controls;

import javafx.beans.property.BooleanProperty;
import javafx.scene.Node;
import javafx.scene.control.Slider;
import javafx.scene.input.MouseEvent;
import lombok.extern.slf4j.Slf4j;

import java.util.Optional;

@Slf4j
public class ProgressSliderControl extends ProgressControl {
    private static final String THUMB_STYLE_CLASS = "thumb";

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
        anchor(slider, true);

        slider.valueProperty().bindBidirectional(timeProperty());
        slider.maxProperty().bind(durationProperty());

        setOnMouseEntered(this::onMouseHover);
        setOnMouseExited(this::onMouseExited);
    }

    private void onMouseHover(MouseEvent event) {
        event.consume();
        sliderThumb().ifPresent(e -> e.setVisible(true));
    }

    private void onMouseExited(MouseEvent event) {
        event.consume();
        sliderThumb().ifPresent(e -> e.setVisible(false));
    }

    private Optional<Node> sliderThumb() {
        return slider.getChildrenUnmodifiable().stream()
                .filter(e -> e.getStyleClass().contains(THUMB_STYLE_CLASS))
                .findFirst();
    }

    //endregion
}
