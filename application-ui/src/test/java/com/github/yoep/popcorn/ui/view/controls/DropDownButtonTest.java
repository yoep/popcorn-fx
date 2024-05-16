package com.github.yoep.popcorn.ui.view.controls;

import com.github.yoep.popcorn.backend.adapters.player.Player;
import javafx.scene.image.Image;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.io.ByteArrayInputStream;
import java.text.MessageFormat;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class DropDownButtonTest {
    @Test
    void testNewInstanceWithFactory() {
        var id = "PlayerId001";
        var player = mock(Player.class);
        var image = new Image(new ByteArrayInputStream(new byte[0]));
        var button = new DropDownButton<Player>(new DropDownButtonFactory<Player>() {
            @Override
            public String getId(Player item) {
                return id;
            }

            @Override
            public String displayName(Player item) {
                return "MyTestPlayer";
            }

            @Override
            public Image graphicResource(Player item) {
                return image;
            }
        });

        button.addDropDownItems(player);

        var result = button.items.get(id);
        assertNotNull(result, MessageFormat.format("expected the item with id {0} to have been found", id));
        assertEquals(player, result.item());
    }

    @Test
    void testSelect_whenItemExists_shouldSelectItem() {
        var id = "PlayerId666";
        var player = mock(Player.class);
        var image = new Image(new ByteArrayInputStream(new byte[0]));
        var button = new DropDownButton<Player>(new DropDownButtonFactory<Player>() {
            @Override
            public String getId(Player item) {
                return id;
            }

            @Override
            public String displayName(Player item) {
                return "MyTestPlayer";
            }

            @Override
            public Image graphicResource(Player item) {
                return image;
            }
        });

        button.addDropDownItems(player);
        button.select(player);

        assertEquals(Optional.of(player), button.getSelectedItem());
    }

    @Test
    void testSelect_whenItemDoesNotExists_shouldNotChangeSelectedItem() {
        var player1Id = "PlayerId666";
        var player1 = mock(Player.class);
        var player2 = mock(Player.class);
        var button = new DropDownButton<Player>(new DropDownButtonFactory<Player>() {
            @Override
            public String getId(Player item) {
                return item.getId();
            }

            @Override
            public String displayName(Player item) {
                return "MyTestPlayer";
            }

            @Override
            public Image graphicResource(Player item) {
                return new Image(new ByteArrayInputStream(new byte[0]));
            }
        });
        when(player1.getId()).thenReturn(player1Id);
        lenient().when(player2.getId()).thenReturn("FooBar");

        button.addDropDownItems(player1);
        button.select(player1);
        button.select(player2);

        assertEquals(Optional.of(player1), button.getSelectedItem());
    }
}