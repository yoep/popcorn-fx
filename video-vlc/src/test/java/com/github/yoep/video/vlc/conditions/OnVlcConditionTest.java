package com.github.yoep.video.vlc.conditions;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.beans.factory.config.ConfigurableListableBeanFactory;
import org.springframework.context.annotation.ConditionContext;
import org.springframework.context.annotation.ConfigurationCondition;
import org.springframework.core.type.AnnotatedTypeMetadata;
import uk.co.caprica.vlcj.factory.discovery.NativeDiscovery;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class OnVlcConditionTest {
    @Mock
    private ConditionContext context;
    @Mock
    private AnnotatedTypeMetadata metadata;
    @InjectMocks
    private OnVlcCondition condition;

    @Test
    void testGetConfigurationPhase_whenInvoked_shouldReturnRegisterBean() {
        var expectedResult = ConfigurationCondition.ConfigurationPhase.REGISTER_BEAN;

        var result = condition.getConfigurationPhase();

        assertEquals(expectedResult, result);
    }

    @Test
    void testMatches_whenBeanFactoryIsNull_shouldReturnFalse() {
        when(context.getBeanFactory()).thenReturn(null);

        var result = condition.matches(context, metadata);

        assertFalse(result);
    }

    @Test
    void testMatches_whenVlcIsNotFound_shouldReturnFalse() {
        var factory = mock(ConfigurableListableBeanFactory.class);
        var discovery = mock(NativeDiscovery.class);
        when(factory.getBean(NativeDiscovery.class)).thenReturn(discovery);
        when(discovery.discover()).thenReturn(false);
        when(context.getBeanFactory()).thenReturn(factory);

        var result = condition.matches(context, metadata);

        assertFalse(result);
    }

    @Test
    void testMatches_whenVlcIsFound_shouldReturnTrue() {
        var factory = mock(ConfigurableListableBeanFactory.class);
        var discovery = mock(NativeDiscovery.class);
        when(factory.getBean(NativeDiscovery.class)).thenReturn(discovery);
        when(discovery.discover()).thenReturn(true);
        when(context.getBeanFactory()).thenReturn(factory);

        var result = condition.matches(context, metadata);

        assertTrue(result);
    }
}
