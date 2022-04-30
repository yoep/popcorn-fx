package com.github.yoep.player.popcorn.controls;

import com.github.spring.boot.javafx.font.controls.Icon;
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

public class Volume extends Icon {
    public static final String VOLUME_PROPERTY = "volume";

    private final DoubleProperty volume = new SimpleDoubleProperty(this, VOLUME_PROPERTY);
    private final VolumePopup popup = new VolumePopup();

    private boolean firstRender = true;

    public Volume() {
        super(Icon.VOLUME_UP_UNICODE);
        init();
    }

    //region Properties

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

            volumeSlider.valueProperty().addListener((observable, oldValue, newValue) -> {
                pane.setMinHeight(volumeSlider.getHeight() * newValue.doubleValue());
                pane.setMaxWidth(volumeSlider.getHeight() * newValue.doubleValue());
            });

            AnchorPane.setRightAnchor(pane, 0.0);
            AnchorPane.setBottomAnchor(pane, 0.0);
            AnchorPane.setLeftAnchor(pane, 0.0);

            return pane;
        }

        private Slider createSlider() {
            var slider = new Slider();
            slider.setOrientation(Orientation.VERTICAL);
            slider.setMin(0.0);
            slider.setMax(1.0);
            return slider;
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
