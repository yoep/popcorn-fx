package com.github.yoep.popcorn.ui.view.controls;

import javafx.beans.property.ObjectProperty;
import javafx.beans.property.ReadOnlyObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.scene.control.MenuItem;
import javafx.scene.control.SplitMenuButton;
import javafx.scene.image.ImageView;
import lombok.extern.slf4j.Slf4j;

import javax.validation.constraints.NotNull;
import java.util.Collection;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.concurrent.ConcurrentHashMap;
import java.util.stream.Collectors;

import static java.util.Arrays.asList;

@Slf4j
public class DropDownButton<T> extends SplitMenuButton {
    public static final String SELECTED_ITEM_PROPERTY = "selectedItem";

    static final String ACTIVE_PLAYER_STYLE_CLASS = "active";

    static final String STYLE_CLASS = "dropdown-button";

    private final ObjectProperty<T> selectedItem = new SimpleObjectProperty<>(this, SELECTED_ITEM_PROPERTY);
    private final Map<Integer, ItemHolder<T>> items = new ConcurrentHashMap<>();

    private DropDownButtonFactory<T> itemFactory;

    public DropDownButton() {
        init();
    }

    public DropDownButton(DropDownButtonFactory<T> itemFactory) {
        this.itemFactory = itemFactory;
        init();
    }

    //region Properties

    /**
     * Get the selected item.
     *
     * @return Returns the selected item, or else {@link Optional#empty()}.
     */
    public Optional<T> getSelectedItem() {
        return Optional.ofNullable(selectedItem.get());
    }

    /**
     * Get the selected item property.
     *
     * @return Returns the selected item property.
     */
    public ReadOnlyObjectProperty<T> selectedItemProperty() {
        return selectedItem;
    }

    /**
     * Get the known players of this button.
     *
     * @return Returns the known players.
     */
    public Collection<T> getDropDownItems() {
        return items.values().stream()
                .map(ItemHolder::item)
                .collect(Collectors.toList());
    }

    /**
     * The item factory for the drop down button.
     *
     * @param itemFactory The item factory of the drop down button.
     */
    public void setItemFactory(@NotNull DropDownButtonFactory<T> itemFactory) {
        Objects.requireNonNull(itemFactory, "itemFactory cannot be null");
        this.itemFactory = itemFactory;
    }

    //endregion

    //region Methods

    /**
     * Clear the current items from the button.
     */
    public void clear() {
        getItems().clear();
        items.clear();

        selectedItem.set(null);
    }

    /**
     * Add the given items to the items.
     *
     * @param items The items to add.
     */
    public void addDropDownItems(Collection<T> items) {
        Objects.requireNonNull(items, "items cannot be null");
        for (T item : items) {
            this.addPlayer(item);
        }
    }

    /**
     * Add the given items to the items.
     *
     * @param items The items to add.
     */
    public void addDropDownItems(T... items) {
        Objects.requireNonNull(items, "items cannot be null");
        addDropDownItems(asList(items));
    }

    /**
     * Select the given item {@link T} in the {@link DropDownButton}.
     * If the item doesn't exist in the drop down items, the select action will be ignored.
     *
     * @param item The {@link T} to select/activate.
     */
    public void select(T item) {
        if (item == null) {
            return;
        }

        var id = item.hashCode();

        if (!isIdActive(id)) {
            updateActivePlayer(id);
        }
    }

    //endregion

    //region Functions

    private void init() {
        getStyleClass().add(STYLE_CLASS);
    }

    private boolean isIdActive(int id) {
        var activeId = getSelectedItem()
                .map(Object::hashCode)
                .orElse(-1);

        return activeId.equals(id);
    }

    private void addPlayer(T player) {
        var id = player.hashCode();
        var control = itemToMenuControlItem(player);
        var holder = new ItemHolder<>(player, control);

        items.put(id, holder);
        getItems().add(control);
    }

    private DropDownMenuItem<T> itemToMenuControlItem(T item) {
        var menuItem = new DropDownMenuItem<>(item, itemFactory);
        menuItem.setOnAction(e -> onPlayerMenuItemSelected(menuItem));
        return menuItem;
    }

    private void updateActivePlayerMenuItem(DropDownMenuItem<T> item) {
        getItems().forEach(e -> e.getStyleClass().removeIf(style -> style.equals(ACTIVE_PLAYER_STYLE_CLASS)));
        item.getStyleClass().add(ACTIVE_PLAYER_STYLE_CLASS);

        Optional.ofNullable(item.getImage())
                .map(ImageView::new)
                .ifPresent(this::setGraphic);
    }

    private void onPlayerMenuItemSelected(MenuItem item) {
        var playerMenuItem = (DropDownMenuItem<T>) item;

        updateActivePlayer(Integer.parseInt(playerMenuItem.getId()));
    }

    private void updateActivePlayer(int id) {
        if (items.containsKey(id)) {
            var holder = items.get(id);

            updateActivePlayerMenuItem(holder.control());
            selectedItem.set(holder.item());
        } else {
            log.warn("Unable to update active drop down item, item \"{}\" not found", id);
        }
    }

    //endregion

    private record ItemHolder<T>(T item, DropDownMenuItem<T> control) {
    }
}
