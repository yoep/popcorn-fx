package com.github.yoep.popcorn.ui.view.controls;

import com.github.yoep.popcorn.backend.info.ComponentState;
import com.github.yoep.popcorn.backend.info.SimpleComponentDetails;
import javafx.application.Platform;
import javafx.scene.input.MouseEvent;
import org.junit.jupiter.api.AfterAll;
import org.junit.jupiter.api.BeforeAll;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.mock;

class AboutCardTest {
    @BeforeAll
    static void beforeAll() {
        Platform.startup(() -> {});
    }

    @AfterAll
    static void afterAll() {
        Platform.exit();
    }

    @Test
    void testInitialize_whenNameIsGiven_shouldSetName() {
        var name = "my-name";
        var details = SimpleComponentDetails.builder()
                .name(name)
                .state(ComponentState.READY)
                .build();
        var card = new AboutCard(details);

        var result = card.nameLabel.getText();

        assertEquals(name, result);
    }

    @Test
    void testOnTitleRowClicked_whenCaretIsClicked_shouldChangeTheExpandedState() {
        var event = mock(MouseEvent.class);
        var details = SimpleComponentDetails.builder()
                .name("lorem")
                .state(ComponentState.READY)
                .build();
        var card = new AboutCard(details);

        card.caretIcon.getOnMouseClicked().handle(event);

        assertTrue(card.getExpanded(), "Expected the card to be expanded");
    }

    @Test
    void testOnTitleRowClicked_whenNameIsClicked_shouldChangeTheExpandedState() {
        var event = mock(MouseEvent.class);
        var details = SimpleComponentDetails.builder()
                .name("lorem")
                .state(ComponentState.READY)
                .build();
        var card = new AboutCard(details);

        card.nameLabel.getOnMouseClicked().handle(event);

        assertTrue(card.getExpanded(), "Expected the card to be expanded");
    }
}