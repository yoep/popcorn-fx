package com.github.yoep.popcorn.subtitle.controls;

import com.github.yoep.popcorn.subtitle.models.SubtitleInfo;
import javafx.application.Platform;
import javafx.beans.property.ListProperty;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.SimpleListProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.collections.FXCollections;
import javafx.collections.ListChangeListener;
import javafx.collections.ObservableList;
import javafx.geometry.Bounds;
import javafx.geometry.Point2D;
import javafx.scene.Node;
import javafx.scene.control.PopupControl;
import javafx.scene.control.Skin;
import javafx.scene.control.Tooltip;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import javafx.scene.layout.FlowPane;
import javafx.scene.layout.StackPane;
import javafx.util.Duration;
import lombok.Getter;
import lombok.extern.slf4j.Slf4j;
import org.springframework.core.io.Resource;
import org.springframework.util.Assert;

import java.io.IOException;
import java.util.ArrayList;
import java.util.List;

/**
 * Control for selecting the subtitle language through flags.
 * These flags are shown in a popup when the control is clicked.
 * <p>
 * The default skin factory used by this control mimics the look of a combobox.
 */
@Slf4j
public class LanguageFlagSelection extends StackPane {
    private static final String STYLE_CLASS = "language-selection";
    private static final String POPUP_STYLE_CLASS = "language-popup";
    private static final String POPUP_IMAGE_STYLE_CLASS = "language-flag";
    private static final int FLAG_WIDTH = 20;
    private static final int FLAG_HEIGHT = 20;

    private final FlagPopup popup = new FlagPopup();

    private final List<LanguageSelectionListener> listeners = new ArrayList<>();
    private final ObjectProperty<SubtitleInfo> selectedItem = new SimpleObjectProperty<>(this, "selectedItem");
    private final ListProperty<SubtitleInfo> items = new SimpleListProperty<>(this, "items", FXCollections.observableArrayList());
    private final ObjectProperty<LanguageFlagCell> factory = new SimpleObjectProperty<>(this, "factory", new LanguageFlagCell());

    private boolean firstRender = true;

    //region Constructors

    public LanguageFlagSelection() {
        init();
    }

    //endregion

    //region Properties

    /**
     * Get the items from this control.
     *
     * @return Returns the control items.
     */
    public ObservableList<SubtitleInfo> getItems() {
        return items.get();
    }

    /**
     * Get the item property of this control.
     *
     * @return Returns the items property.
     */
    public ListProperty<SubtitleInfo> itemsProperty() {
        return items;
    }

    /**
     * Set the items for this control.
     *
     * @param items The new items of this control.
     */
    public void setItems(ObservableList<SubtitleInfo> items) {
        this.items.set(items);
    }

    /**
     * Get the current selected item in the control.
     *
     * @return Returns the selected item.
     */
    public SubtitleInfo getSelectedItem() {
        return selectedItem.get();
    }

    /**
     * Get the selected item property of this control.
     *
     * @return Returns the selected item property.
     */
    public ObjectProperty<SubtitleInfo> selectedItemProperty() {
        return selectedItem;
    }

    /**
     * Set the selected item of this control.
     *
     * @param selectedItem The item that should be selected in this control.
     */
    public void setSelectedItem(SubtitleInfo selectedItem) {
        this.selectedItem.set(selectedItem);
    }

    /**
     * Get the factory of this control.
     *
     * @return Returns the factory used by this control.
     */
    public LanguageFlagCell getFactory() {
        return factory.get();
    }

    /**
     * Get the factory property if this control.
     *
     * @return Returns the factory property.
     */
    public ObjectProperty<LanguageFlagCell> factoryProperty() {
        return factory;
    }

    /**
     * Set the factory that should be used by this control.
     *
     * @param factory The factory to use.
     */
    public void setFactory(LanguageFlagCell factory) {
        Assert.notNull(factory, "factory cannot be null");
        this.factory.set(factory);
    }

    //endregion

    //region Methods

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
        onSelectedItemChanged(items.get(index));
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

    //endregion

    //region Functions

    /**
     * Update this control with the selected item.
     *
     * @param newValue The item that has been selected.
     */
    protected void onSelectedItemChanged(final SubtitleInfo newValue) {
        // always hide the popup when an item has been clicked in the popup
        Platform.runLater(popup::hide);

        if (getSelectedItem() == newValue)
            return;

        setSelectedItem(newValue);
        updateFactorySkin();

        synchronized (listeners) {
            listeners.forEach(e -> e.onItemChanged(newValue));
        }
    }

    private void init() {
        initializePopup();
        initializeEvents();
        initializeFactory();

        getStyleClass().add(STYLE_CLASS);
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

    private void initializeFactory() {
        updateFactory(getFactory());
        factoryProperty().addListener((observable, oldValue, newValue) -> updateFactory(newValue));
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
        Resource flagResource = subtitle.getFlagResource();
        Flag flag = new Flag(subtitle);

        flag.getStyleClass().add(POPUP_IMAGE_STYLE_CLASS);
        flag.setOnMouseClicked(event -> onSelectedItemChanged(subtitle));

        Tooltip tooltip = new Tooltip(subtitle.getLanguage().getNativeName());
        tooltip.setShowDelay(Duration.ZERO);
        Tooltip.install(flag, tooltip);

        popup.getContent().add(flag);
        loadImage(flag.getImageView(), flagResource);
    }

    private void removeFlag(SubtitleInfo subtitle) {
        popup.getContent().removeIf(e -> ((Flag) e).getSubtitle() == subtitle);
    }

    private void loadImage(ImageView imageView, Resource imageResource) {
        try {
            imageView.setImage(new Image(imageResource.getInputStream()));
        } catch (IOException ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    private void updateFactory(LanguageFlagCell newValue) {
        newValue.setOnMouseClicked(event -> onClicked());
        Platform.runLater(() -> {
            getChildren().clear();
            getChildren().add(newValue);
        });
    }

    private void updateFactorySkin() {
        getFactory().updateItem(getSelectedItem());
    }

    //endregion

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
