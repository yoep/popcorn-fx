package com.github.yoep.video.youtube.conditions;

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
class OnYoutubeVideoEnabledTest {
    @Mock
    private ConditionContext context;
    @Mock
    private AnnotatedTypeMetadata metadata;
    @Mock
    private ConfigurableListableBeanFactory beanFactory;
    @Mock
    private ApplicationArguments applicationArguments;
    @InjectMocks
    private OnYoutubeVideoEnabled condition;

    @Test
    void testGetConfigurationPhase_whenInvoked_shouldReturnRegisterPhase() {
        var result = condition.getConfigurationPhase();

        assertEquals(ConfigurationCondition.ConfigurationPhase.REGISTER_BEAN, result);
    }

    @Test
    void testMatches_whenDisableOptionIsNotPresent_shouldReturnTrue() {
        when(context.getBeanFactory()).thenReturn(beanFactory);
        when(beanFactory.getBean(ApplicationArguments.class)).thenReturn(applicationArguments);
        when(applicationArguments.containsOption(OnYoutubeVideoEnabled.DISABLE_OPTION)).thenReturn(false);

        var result = condition.matches(context, metadata);

        assertTrue(result);
    }

    @Test
    void testMatches_whenBeanFactoryIsNotPresent_shouldReturnTrue() {
        when(context.getBeanFactory()).thenReturn(null);

        var result = condition.matches(context, metadata);

        assertTrue(result);
    }

    @Test
    void testMatches_whenDisableOptionIsPresent_shouldReturnFalse() {
        when(context.getBeanFactory()).thenReturn(beanFactory);
        when(beanFactory.getBean(ApplicationArguments.class)).thenReturn(applicationArguments);
        when(applicationArguments.containsOption(OnYoutubeVideoEnabled.DISABLE_OPTION)).thenReturn(true);

        var result = condition.matches(context, metadata);

        assertFalse(result);
    }
}