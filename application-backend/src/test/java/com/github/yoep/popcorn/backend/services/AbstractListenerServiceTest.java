package com.github.yoep.popcorn.backend.services;

import org.junit.jupiter.api.Test;

import java.util.List;

import static java.util.Arrays.asList;
import static org.junit.jupiter.api.Assertions.assertEquals;

class AbstractListenerServiceTest {
    @Test
    void testAddListener_whenListenerISGiven_shouldAddListenerToList() {
        var listener = new TestListener() {
            @Override
            public void onChange() {
            }
        };
        var service = new TestService();

        service.addListener(listener);
        var result = service.getListeners();

        assertEquals(1, result.size());
        assertEquals(listener, result.get(0));
    }

    @Test
    void testRemoveListener_whenListenerIsGiven_shouldRemoveTheListener() {
        var listener = new TestListener() {
            @Override
            public void onChange() {
            }
        };
        var service = new TestService();

        service.addListener(listener);
        service.removeListener(listener);
        var result = service.getListeners();

        assertEquals(0, result.size());
    }

    interface TestListener {
        void onChange();
    }

    class TestService extends AbstractListenerService<TestListener> {
        public List<TestListener> getListeners() {
            return asList(listeners.toArray(new TestListener[0]));
        }
    }
}