package com.github.yoep.player.qt.conditions;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.beans.factory.config.ConfigurableListableBeanFactory;
import org.springframework.boot.ApplicationArguments;
import org.springframework.context.annotation.ConditionContext;
import org.springframework.context.annotation.ConfigurationCondition;
import org.springframework.core.type.AnnotatedTypeMetadata;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class OnQtPlayerEnabledConditionTest {
    @Mock
    private ConditionContext context;
    @Mock
    private AnnotatedTypeMetadata metadata;
    @Mock
    private ConfigurableListableBeanFactory beanFactory;
    @Mock
    private ApplicationArguments arguments;
    @InjectMocks
    private OnQtPlayerEnabledCondition condition;

    @Test
    void testGetConfigurationPhase_whenInvoked_shouldReturnRegisterPhase() {
        var result = condition.getConfigurationPhase();

        assertEquals(ConfigurationCondition.ConfigurationPhase.REGISTER_BEAN, result);
    }

    @Test
    void testMatches_whenBeanFactoryIsNull_shouldReturnTrue() {
        var result = condition.matches(context, metadata);

        assertTrue(result, "Expected the condition to match by default");
    }

    @Test
    void testMatches_whenDisableOptionIsNotPresent_shouldReturnTrue() {
        when(context.getBeanFactory()).thenReturn(beanFactory);
        when(beanFactory.getBean(ApplicationArguments.class)).thenReturn(arguments);
        when(arguments.containsOption(OnQtPlayerEnabledCondition.DISABLE_OPTION)).thenReturn(false);

        var result = condition.matches(context, metadata);

        assertTrue(result, "Expected the condition to match when disable option is not present");
    }

    @Test
    void testMatches_whenDisableOptionIsPresent_shouldReturnFalse() {
        when(context.getBeanFactory()).thenReturn(beanFactory);
        when(beanFactory.getBean(ApplicationArguments.class)).thenReturn(arguments);
        when(arguments.containsOption(OnQtPlayerEnabledCondition.DISABLE_OPTION)).thenReturn(true);

        var result = condition.matches(context, metadata);

        assertFalse(result, "Expected the condition to not match when disable option is present");
    }
}