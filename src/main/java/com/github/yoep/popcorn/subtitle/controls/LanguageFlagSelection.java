package com.github.yoep.popcorn.subtitle.controls;

import com.github.spring.boot.javafx.font.controls.Icon;
import com.github.yoep.popcorn.subtitle.models.SubtitleInfo;
import javafx.collections.FXCollections;
import javafx.collections.ListChangeListener;
import javafx.collections.ObservableList;
import javafx.geometry.Bounds;
import javafx.geometry.Point2D;
import javafx.geometry.Pos;
import javafx.scene.Node;
import javafx.scene.control.PopupControl;
import javafx.scene.control.Skin;
import javafx.scene.control.Tooltip;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.layout.FlowPane;
import javafx.scene.layout.HBox;
import javafx.scene.layout.StackPane;
import javafx.util.Duration;
import lombok.Getter;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.Resource;
import org.springframework.util.Assert;

import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

/**
 * Control for selecting the subtitle language through flags.
 * These flags are shown in a popup when the control is clicked.
 * <p>
 * The control itself mimics the look of a combobox.
 */
@Slf4j
public class LanguageFlagSelection extends HBox {
    private static final String STYLE_CLASS = "language-selection";
    private static final String ITEM_STYLE_CLASS = "item";
    private static final String ARROW_STYLE_CLASS = "arrow";
    private static final String POPUP_STYLE_CLASS = "language-popup";
    private static final String POPUP_IMAGE_STYLE_CLASS = "language-flag";
    private static final int FLAG_WIDTH = 20;
    private static final int FLAG_HEIGHT = 20;

    private final ImageView imageView = new ImageView();
    private final Icon arrow = new Icon(Icon.CARET_UP_UNICODE);
    private final FlagPopup popup = new FlagPopup();

    private final List<LanguageSelectionListener> listeners = new ArrayList<>();
    private final ObservableList<SubtitleInfo> items = FXCollections.observableArrayList();

    private SubtitleInfo selectedItem;
    private boolean firstRender = true;

    public LanguageFlagSelection() {
        init();
    }

    /**
     * Get the items of this language selection.
     *
     * @return Returns the items of this instance.
     */
    public ObservableList<SubtitleInfo> getItems() {
        return items;
    }

    /**
     * Get the selected item of this language selection.
     *
     * @return Returns the selected item if present, else {@link Optional#empty()}.
     */
    public Optional<SubtitleInfo> getSelectedItem() {
        return Optional.ofNullable(selectedItem);
    }

    public void addListener(LanguageSelectionListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.add(listener);
        }
    }

    public void removeListener(LanguageSelectionListener listener) {
        Assert.notNull(listener, "listener cannot be null");
        synchronized (listeners) {
            listeners.remove(listener);
        }
    }

    /**
     * Select the given item index.
     *
     * @param index The index of the item to select.
     */
    public void select(int index) {
        selectItem(items.get(index));
    }

    /**
     * Show the language selection popup of this control.
     */
    public void show() {
        Point2D position = calculatePopupPosition();

        popup.show(this, position.getX(), position.getY());

        if (firstRender) {
            firstRender = false;
            show();
        }
    }

    private void init() {
        initializeImageView();
        initializeArrow();
        initializePopup();
        initializeEvents();

        this.getStyleClass().add(STYLE_CLASS);
        this.setAlignment(Pos.CENTER);
    }

    private void initializeImageView() {
        imageView.getStyleClass().add(ITEM_STYLE_CLASS);
        imageView.setPreserveRatio(true);
        imageView.fitHeightProperty().bind(heightProperty());

        getChildren().add(imageView);
    }

    private void initializeArrow() {
        arrow.getStyleClass().add(ARROW_STYLE_CLASS);

        getChildren().add(arrow);
    }

    private void initializePopup() {
        popup.getContentPane().getStyleClass().add(POPUP_STYLE_CLASS);

        popup.setAutoHide(true);
        popup.setAutoFix(true);

        popup.getContentPane().heightProperty().addListener((observable, oldValue, newValue) -> movePopup());
        popup.getContentPane().widthProperty().addListener((observable, oldValue, newValue) -> movePopup());
    }

    private void initializeEvents() {
        this.setOnMouseClicked(event -> onClicked());
        items.addListener((ListChangeListener<SubtitleInfo>) change -> {
            while (change.next()) {
                if (change.wasAdded()) {
                    change.getAddedSubList().forEach(this::addNewFlag);
                } else if (change.wasRemoved()) {
                    change.getRemoved().forEach(this::removeFlag);
                }
            }
        });
    }

    private void onClicked() {
        if (popup.isShowing()) {
            popup.hide();
        } else {
            show();
        }
    }

    private Point2D calculatePopupPosition() {
        Bounds screenBounds = this.localToScreen(this.getBoundsInLocal());
        double x = screenBounds.getMaxX() - popup.getContentPane().getWidth();
        double y = screenBounds.getMinY() - popup.getContentPane().getHeight();

        return new Point2D(x, y);
    }

    private void movePopup() {
        Point2D position = calculatePopupPosition();

        popup.setAnchorX(position.getX());
        popup.setAnchorY(position.getY());
    }

    private void addNewFlag(final SubtitleInfo subtitle) {
        subtitle.getFlagResource().ifPresent(e -> {
            Flag flag = new Flag(subtitle);

            flag.getStyleClass().add(POPUP_IMAGE_STYLE_CLASS);
            flag.setOnMouseClicked(event -> selectItem(subtitle));

            Tooltip tooltip = new Tooltip(subtitle.getLanguage());
            tooltip.setShowDelay(Duration.ZERO);
            Tooltip.install(flag, tooltip);

            popup.getContent().add(flag);
            loadImage(flag.getImageView(), e);
        });
    }

    private void removeFlag(SubtitleInfo subtitle) {
        popup.getContent().removeIf(e -> ((Flag) e).getSubtitle() == subtitle);
    }

    private void selectItem(final SubtitleInfo subtitle) {
        if (selectedItem == subtitle)
            return;

        selectedItem = subtitle;
        popup.hide();

        subtitle.getFlagResource().ifPresent(e -> loadImage(this.imageView, e));
        Tooltip tooltip = new Tooltip(subtitle.getLanguage());
        tooltip.setShowDelay(Duration.ZERO);
        Tooltip.install(this.imageView, tooltip);

        synchronized (listeners) {
            listeners.forEach(e -> e.onItemChanged(subtitle));
        }
    }

    private void loadImage(ImageView imageView, Resource imageResource) {
        try {
            imageView.setImage(new Image(imageResource.getInputStream()));
        } catch (IOException ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    @Getter
    private static class Flag extends StackPane {
        private final ImageView imageView = new ImageView();
        private final SubtitleInfo subtitle;

        private Flag(SubtitleInfo subtitle) {
            this.subtitle = subtitle;

            init();
        }

        public ImageView getImageView() {
            return imageView;
        }

        private void init() {
            imageView.setFitHeight(FLAG_HEIGHT);
            imageView.setFitWidth(FLAG_WIDTH);
            imageView.setPreserveRatio(true);

            this.getChildren().add(imageView);
        }
    }

    private static class FlagPopup extends PopupControl {
        private final FlowPane content = new FlowPane();

        public FlagPopup() {
            init();
        }

        /**
         * Get the content of this flag popup.
         *
         * @return Returns the content nodes of this popup.
         */
        ObservableList<Node> getContent() {
            return content.getChildren();
        }

        FlowPane getContentPane() {
            return content;
        }

        @Override
        protected Skin<?> createDefaultSkin() {
            return new FlagPopupSkin(this, content);
        }

        private void init() {
            bridge.getChildren().add(content);
        }
    }

    private static class FlagPopupSkin implements Skin<FlagPopup> {
        private final FlagPopup flagPopup;
        private final FlowPane contentPane;

        public FlagPopupSkin(FlagPopup flagPopup, FlowPane content) {
            this.flagPopup = flagPopup;
            contentPane = content;
        }

        @Override
        public FlagPopup getSkinnable() {
            return flagPopup;
        }

        @Override
        public Node getNode() {
            return contentPane;
        }

        @Override
        public void dispose() {

        }
    }
}
