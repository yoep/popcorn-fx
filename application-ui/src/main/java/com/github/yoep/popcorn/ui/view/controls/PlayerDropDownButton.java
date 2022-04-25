package com.github.yoep.popcorn.ui.view.controls;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import javafx.scene.image.Image;
import lombok.extern.slf4j.Slf4j;

import java.io.IOException;

@Slf4j
public class PlayerDropDownButton extends DropDownButton<Player> {
    public PlayerDropDownButton() {
        super(createDropDownButtonFactory());
    }

    private static DropDownButtonFactory<Player> createDropDownButtonFactory() {
        return new DropDownButtonFactory<>() {
            @Override
            public String displayName(Player item) {
                return item.getName();
            }

            @Override
            public Image graphicResource(Player item) {
                return item.getGraphicResource()
                        .map(resource -> {
                            try {
                                return new Image(resource.getInputStream(), 16, 16, true, true);
                            } catch (IOException ex) {
                                log.error("Failed to create image from graphic resource, {}", ex.getMessage(), ex);
                                return null;
                            }
                        }).orElse(null);
            }
        };
    }
}
