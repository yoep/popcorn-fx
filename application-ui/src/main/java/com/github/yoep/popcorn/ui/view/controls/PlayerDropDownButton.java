package com.github.yoep.popcorn.ui.view.controls;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import javafx.scene.image.Image;
import lombok.extern.slf4j.Slf4j;

@Slf4j
public class PlayerDropDownButton extends DropDownButton<Player> {
    public PlayerDropDownButton() {
        super(createDropDownButtonFactory());
    }

    private static DropDownButtonFactory<Player> createDropDownButtonFactory() {
        return new DropDownButtonFactory<>() {
            @Override
            public String getId(Player item) {
                return item.getId();
            }

            @Override
            public String displayName(Player item) {
                return item.getName();
            }

            @Override
            public Image graphicResource(Player item) {
                return item.getGraphicResource()
                        .map(resource -> new Image(resource, 16, 16, true, true))
                        .orElse(null);
            }
        };
    }
}
