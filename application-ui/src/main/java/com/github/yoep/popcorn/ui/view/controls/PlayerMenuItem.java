package com.github.yoep.popcorn.ui.view.controls;

import com.github.yoep.player.adapter.Player;
import javafx.scene.Node;
import javafx.scene.control.MenuItem;
import javafx.scene.image.Image;
import javafx.scene.image.ImageView;
import lombok.EqualsAndHashCode;
import lombok.extern.slf4j.Slf4j;

import java.io.IOException;
import java.util.Optional;

@Slf4j
@EqualsAndHashCode(callSuper = false)
public class PlayerMenuItem extends MenuItem {
    private final Player player;
    private final Image image;

    public PlayerMenuItem(Player player) {
        super(player.getName());
        this.player = player;
        this.image = createImageFromGraphicResource();

        setId(player.getId());
        setGraphic(createGraphicNode());
    }

    public Image getImage() {
        return image;
    }

    private Image createImageFromGraphicResource() {
        return player.getGraphicResource()
                .map(resource -> {
                    try {
                        return new Image(resource.getInputStream(), 16, 16, true, true);
                    } catch (IOException ex) {
                        log.error("Failed to create image from graphic resource, {}", ex.getMessage(), ex);
                        return null;
                    }
                }).orElse(null);
    }

    private Node createGraphicNode() {
        return Optional.ofNullable(image)
                .map(ImageView::new)
                .orElse(null);
    }
}
