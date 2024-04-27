package com.github.yoep.player.popcorn.controls;

import com.github.yoep.popcorn.ui.font.controls.IconSolid;
import javafx.application.Platform;
import javafx.beans.property.DoubleProperty;
import javafx.beans.property.SimpleDoubleProperty;
import javafx.geometry.Insets;
import javafx.geometry.Orientation;
import javafx.scene.Node;
import javafx.scene.control.PopupControl;
import javafx.scene.control.Skin;
import javafx.scene.control.Slider;
import javafx.scene.input.MouseEvent;
import javafx.scene.layout.AnchorPane;
import javafx.scene.layout.Pane;
import javafx.scene.layout.StackPane;

public class Volume extends IconSolid {
    public static final String VOLUME_PROPERTY = "volume";
    public static final String VOLUME_HIGH_UNICODE = IconSolid.VOLUME_UP_UNICODE;
    public static final String VOLUME_UNICODE = IconSolid.VOLUME_DOWN_UNICODE;
    public static final String VOLUME_LOW_UNICODE = IconSolid.VOLUME_OFF_UNICODE;
    public static final String VOLUME_XMARK_UNICODE = IconSolid.VOLUME_MUTE_UNICODE;

    /**
     * The volume value between 0 and 1.
     */
    private final DoubleProperty volume = new SimpleDoubleProperty(this, VOLUME_PROPERTY);
    private final VolumePopup popup = new VolumePopup();

    private boolean firstRender = true;
    private boolean isValueChanging;

    public Volume() {
        super(VOLUME_HIGH_UNICODE);
        init();
    }

    //region Properties

    public boolean isValueChanging() {
        return isValueChanging;
    }

    public double getVolume() {
        return volume.get();
    }

    public DoubleProperty volumeProperty() {
        return volume;
    }

    public void setVolume(double volume) {
        this.volume.set(volume);
    }

    //endregion

    private void init() {
        popup.setAutoHide(true);
        popup.setAutoFix(true);
        setOnMouseClicked(this::onClicked);
        volume.addListener((observable, oldValue, newValue) -> onVolumeChanged(newValue.doubleValue()));
    }

    private void onClicked(MouseEvent event) {
        event.consume();

        if (popup.isShowing()) {
            popup.hide();
        } else {
            onShowPopup();
        }
    }

    private void onShowPopup() {
        var screenBounds = this.localToScreen(this.getBoundsInLocal());
        var x = screenBounds.getMaxX();
        var y = screenBounds.getMinY();

        popup.show(this, x - popup.getContent().getWidth(), y - popup.getContent().getHeight());

        if (firstRender) {
            firstRender = false;
            onShowPopup();
        }
    }

    private void onVolumeChanged(double value) {
        Platform.runLater(() -> {
            if (value == 0) {
                setText(VOLUME_XMARK_UNICODE);
            } else if (value <= 0.33) {
                setText(VOLUME_LOW_UNICODE);
            } else if (value > 0.33 && value < 0.66) {
                setText(VOLUME_UNICODE);
            } else {
                setText(VOLUME_HIGH_UNICODE);
            }
        });
    }

    private class VolumePopup extends PopupControl {
        private final VolumePopupSkin skin = new VolumePopupSkin(this);

        @Override
        protected Skin<?> createDefaultSkin() {
            return skin;
        }

        /**
         * Get the content node of the popup.
         *
         * @return Returns the content node.
         */
        Pane getContent() {
            return (Pane) skin.getNode();
        }
    }

    private class VolumePopupSkin implements Skin<VolumePopup> {
        private static final String CONTENT_STYLE_CLASS = "volume-popup";
        private static final String SLIDER_STYLE_CLASS = "volume-slider";
        private static final String BACKGROUND_TRACK_STYLE_CLASS = "background-track";
        private static final String VOLUME_TRACK_STYLE_CLASS = "volume-track";

        private final VolumePopup volumePopup;

        private StackPane content;
        private AnchorPane enhancedSlider;
        private Slider volumeSlider;
        private StackPane backgroundTrackPane;
        private StackPane volumeTrackPane;

        public VolumePopupSkin(VolumePopup volumePopup) {
            this.volumePopup = volumePopup;
            init();
        }

        @Override
        public VolumePopup getSkinnable() {
            return volumePopup;
        }

        @Override
        public Node getNode() {
            return content;
        }

        @Override
        public void dispose() {
            this.content = null;
            this.enhancedSlider = null;
            this.volumeSlider = null;
            this.backgroundTrackPane = null;
            this.volumeTrackPane = null;
        }

        private void init() {
            content = createContent();
            enhancedSlider = createEnhancedSlider();
            volumeSlider = createSlider();
            backgroundTrackPane = createTrackPane();
            volumeTrackPane = createVolumeTrack();

            enhancedSlider.getChildren().addAll(anchor(backgroundTrackPane), volumeTrackPane, anchor(volumeSlider));
            content.getChildren().add(enhancedSlider);
        }

        private StackPane createContent() {
            var pane = new StackPane();
            pane.setPadding(new Insets(5));
            pane.getStyleClass().add(CONTENT_STYLE_CLASS);
            return pane;
        }

        private AnchorPane createEnhancedSlider() {
            var pane = new AnchorPane();
            pane.getStyleClass().add(SLIDER_STYLE_CLASS);
            return pane;
        }

        private StackPane createTrackPane() {
            var pane = new StackPane();
            pane.getStyleClass().add(BACKGROUND_TRACK_STYLE_CLASS);

            volumeSlider.widthProperty().addListener((observable, oldValue, newValue) -> pane.setPrefWidth(newValue.doubleValue()));
            volumeSlider.heightProperty().addListener((observable, oldValue, newValue) -> pane.setPrefHeight(newValue.doubleValue()));

            return pane;
        }

        private StackPane createVolumeTrack() {
            var pane = new StackPane();
            pane.getStyleClass().add(VOLUME_TRACK_STYLE_CLASS);

            volumeSlider.heightProperty().addListener(observable -> updateVolumeTrack());
            volume.addListener(observable -> updateVolumeTrack());

            AnchorPane.setRightAnchor(pane, 0.0);
            AnchorPane.setBottomAnchor(pane, 0.0);
            AnchorPane.setLeftAnchor(pane, 0.0);

            return pane;
        }

        private Slider createSlider() {
            var slider = new Slider();
            slider.valueProperty().bindBidirectional(volume);
            slider.valueChangingProperty().addListener((observable, oldValue, newValue) -> isValueChanging = newValue);
            slider.setOrientation(Orientation.VERTICAL);
            slider.setMin(0.0);
            slider.setMax(1.0);
            return slider;
        }

        private void updateVolumeTrack() {
            volumeTrackPane.setMinHeight(volumeSlider.getHeight() * volume.get());
            volumeTrackPane.setMaxWidth(volumeSlider.getHeight() * volume.get());
        }

        private <T extends Node> T anchor(T node) {
            AnchorPane.setTopAnchor(node, 0.0);
            AnchorPane.setRightAnchor(node, 0.0);
            AnchorPane.setBottomAnchor(node, 0.0);
            AnchorPane.setLeftAnchor(node, 0.0);
            return node;
        }
    }
}
