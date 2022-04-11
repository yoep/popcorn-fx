package com.github.yoep.popcorn.ui.view.controls;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import javafx.beans.property.ObjectProperty;
import javafx.beans.property.ReadOnlyObjectProperty;
import javafx.beans.property.SimpleObjectProperty;
import javafx.scene.control.MenuItem;
import javafx.scene.control.SplitMenuButton;
import javafx.scene.image.ImageView;
import lombok.Getter;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.util.Assert;

import java.util.Collection;
import java.util.HashMap;
import java.util.Map;
import java.util.Optional;
import java.util.stream.Collectors;

@Slf4j
public class WatchNowButton extends SplitMenuButton {
    public static final String SELECTED_ITEM_PROPERTY = "selectedItem";
    private static final String ACTIVE_PLAYER_STYLE_CLASS = "active";
    private static final String STYLE_CLASS = "watch-now";

    private final ObjectProperty<Player> selectedItem = new SimpleObjectProperty<>(this, SELECTED_ITEM_PROPERTY);
    private final Map<String, PlayerHolder> players = new HashMap<>();

    public WatchNowButton() {
        init();
    }

    //region Properties

    /**
     * Get the selected item.
     *
     * @return Returns the selected item, or else {@link Optional#empty()}.
     */
    public Optional<Player> getSelectedItem() {
        return Optional.ofNullable(selectedItem.get());
    }

    /**
     * Get the selected item property.
     *
     * @return Returns the selected item property.
     */
    public ReadOnlyObjectProperty<Player> selectedItemProperty() {
        return selectedItem;
    }

    /**
     * Get the known players of this button.
     *
     * @return Returns the known players.
     */
    public Collection<Player> getPlayers() {
        return players.values().stream()
                .map(PlayerHolder::getPlayer)
                .collect(Collectors.toList());
    }

    //endregion

    //region Methods

    /**
     * Clear the current items from the button.
     */
    public void clear() {
        getItems().clear();
        players.clear();

        selectedItem.set(null);
    }

    /**
     * Add the player to the items.
     *
     * @param player The player to add.
     */
    public void addItem(Player player) {
        Assert.notNull(player, "player cannot be null");
        addPlayer(player);
    }

    /**
     * Add the given players to the items.
     *
     * @param players The players to add.
     */
    public void addItems(Collection<Player> players) {
        Assert.notNull(players, "players cannot be null");
        players.forEach(this::addPlayer);
    }

    /**
     * Select the given {@link Player} in the {@link WatchNowButton}.
     * If the player doesn't exist in the items, the select action will be ignored.
     *
     * @param player The {@link Player} to select/activate.
     */
    public void select(Player player) {
        if (player == null) {
            return;
        }

        var id = player.getId();

        if (!isIdActive(id)) {
            updateActivePlayer(id);
        }
    }

    //endregion

    //region Functions

    private void init() {
        getStyleClass().add(STYLE_CLASS);
    }

    private boolean isIdActive(String id) {
        var activeId = getSelectedItem()
                .map(Player::getId)
                .orElse("");

        return activeId.equals(id);
    }

    private void addPlayer(Player player) {
        var id = player.getId();
        var control = playerToMenuItem(player);
        var holder = new PlayerHolder(player, control);

        players.put(id, holder);
        getItems().add(control);
    }

    private PlayerMenuItem playerToMenuItem(Player player) {
        var item = new PlayerMenuItem(player);
        item.setOnAction(e -> onPlayerMenuItemSelected(item));
        return item;
    }

    private void updateActivePlayerMenuItem(PlayerMenuItem item) {
        getItems().forEach(e -> e.getStyleClass().removeIf(style -> style.equals(ACTIVE_PLAYER_STYLE_CLASS)));
        item.getStyleClass().add(ACTIVE_PLAYER_STYLE_CLASS);

        Optional.ofNullable(item.getImage())
                .map(ImageView::new)
                .ifPresent(this::setGraphic);
    }

    private void onPlayerMenuItemSelected(MenuItem item) {
        var playerMenuItem = (PlayerMenuItem) item;
        var id = playerMenuItem.getId();

        updateActivePlayer(id);
    }

    private void updateActivePlayer(String id) {
        if (players.containsKey(id)) {
            var holder = players.get(id);

            updateActivePlayerMenuItem(holder.getControl());
            selectedItem.set(holder.getPlayer());
        } else {
            log.warn("Unable to update active player, player ID \"{}\" not found", id);
        }
    }

    //endregion

    @Getter
    @RequiredArgsConstructor
    private static class PlayerHolder {
        private final Player player;
        private final PlayerMenuItem control;
    }
}
