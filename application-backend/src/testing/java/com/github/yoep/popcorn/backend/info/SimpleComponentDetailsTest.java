package testing.java.com.github.yoep.popcorn.backend.info;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.beans.PropertyChangeEvent;
import java.beans.PropertyChangeListener;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.lenient;

@ExtendWith(MockitoExtension.class)
class SimpleComponentDetailsTest {
    @Mock
    private PropertyChangeListener listener;

    private final AtomicReference<PropertyChangeEvent> eventHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        lenient().doAnswer(invocation -> {
            eventHolder.set(invocation.getArgument(0, PropertyChangeEvent.class));
            return null;
        }).when(listener).propertyChange(isA(PropertyChangeEvent.class));
    }

    @Test
    void testGetDescription_whenDescriptionIsNull_shouldReturnEmpty() {
        var details = SimpleComponentDetails.builder()
                .name("lorem")
                .state(ComponentState.ERROR)
                .build();

        var result = details.getDescription();

        assertNotNull(result, "Expected an optional description to have been returned");
        assertTrue(result.isEmpty(), "Expected the description to be empty");
    }

    @Test
    void testDescriptionProperty_whenDescriptionIsChanged_shouldFirePropertyChange() {
        var details = SimpleComponentDetails.builder()
                .name("lorem")
                .build();
        var description = "my new description";
        details.addChangeListener(listener);

        details.setDescription(description);
        var result = eventHolder.get();

        assertNotNull(result, "Expected an event to have been fired");
        assertEquals(ComponentInfo.DESCRIPTION_PROPERTY, result.getPropertyName());
        assertEquals(description, result.getNewValue());
    }

    @Test
    void testStateProperty_whenStateIsChanged_shouldFirePropertyChange() {
        var details = SimpleComponentDetails.builder()
                .name("lorem")
                .build();
        var state = ComponentState.READY;
        details.addChangeListener(listener);

        details.setState(state);
        var result = eventHolder.get();

        assertNotNull(result, "Expected an event to have been fired");
        assertEquals(ComponentInfo.STATE_PROPERTY, result.getPropertyName());
        assertEquals(state, result.getNewValue());
    }
}