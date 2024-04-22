package com.github.yoep.popcorn.ui.view.controls;

import com.github.yoep.popcorn.ui.font.controls.Icon;
import javafx.animation.Animation;
import javafx.animation.FadeTransition;
import javafx.beans.property.BooleanProperty;
import javafx.beans.property.SimpleBooleanProperty;
import javafx.util.Duration;

public class HealthIcon extends Icon {
    public static final String UPDATING_PROPERTY = "updating";

    private final FadeTransition animation = new FadeTransition(Duration.seconds(2), this);
    private final BooleanProperty updating = new SimpleBooleanProperty(this, UPDATING_PROPERTY, false);

    public HealthIcon() {
        super(CIRCLE_UNICODE);
        init();
    }

    //region Properties

    public boolean isUpdating() {
        return updating.get();
    }

    public BooleanProperty updatingProperty() {
        return updating;
    }

    public void setUpdating(boolean updating) {
        this.updating.set(updating);
    }

    //endregion

    //region Functions

    private void init() {
        initializeAnimation();
        initializeListeners();
    }

    private void initializeAnimation() {
        animation.setFromValue(0.5);
        animation.setToValue(1);
        animation.setCycleCount(Animation.INDEFINITE);
    }

    private void initializeListeners() {
        updating.addListener((observable, oldValue, newValue) -> {
            if (newValue) {
                animation.play();
            } else {
                animation.stop();
                this.setOpacity(1.0);
            }
        });
    }

    //endregion
}
