package com.github.yoep.popcorn.backend.settings.models;

import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.beans.PropertyChangeEvent;
import java.beans.PropertyChangeListener;
import java.lang.reflect.InvocationTargetException;
import java.util.concurrent.atomic.AtomicReference;

import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.doAnswer;

@ExtendWith(MockitoExtension.class)
public abstract class AbstractPropertyTest<T extends AbstractSettings> {
    private final Class<T> classType;

    @Mock
    PropertyChangeListener listener;
    T settings;

    private final AtomicReference<String> propertyNameHolder = new AtomicReference<>();
    private final AtomicReference<Object> newValueHolder = new AtomicReference<>();

    protected AbstractPropertyTest(Class<T> classType) {
        this.classType = classType;
    }

    @BeforeEach
    void setUp() throws NoSuchMethodException, InvocationTargetException, InstantiationException, IllegalAccessException {
        settings = classType.getConstructor().newInstance();
        settings.addListener(listener);
        doAnswer(invocationOnMock -> {
            var changeEvent = invocationOnMock.getArgument(0, PropertyChangeEvent.class);
            propertyNameHolder.set(changeEvent.getPropertyName());
            newValueHolder.set(changeEvent.getNewValue());
            return null;
        }).when(listener).propertyChange(isA(PropertyChangeEvent.class));
    }

    @AfterEach
    void tearDown() {
        propertyNameHolder.set(null);
        newValueHolder.set(null);
    }

    protected String invokedPropertyName() {
        return propertyNameHolder.get();
    }

    protected Object invokedNewValue() {
        return newValueHolder.get();
    }
}
