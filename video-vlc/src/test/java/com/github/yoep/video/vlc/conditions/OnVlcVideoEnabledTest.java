package com.github.yoep.video.vlc.conditions;

import org.junit.jupiter.api.BeforeEach;
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
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class OnVlcVideoEnabledTest {
    @Mock
    private ConditionContext context;
    @Mock
    private AnnotatedTypeMetadata metadata;
    @Mock
    private ConfigurableListableBeanFactory beanFactory;
    @Mock
    private ApplicationArguments applicationArguments;
    @InjectMocks
    private OnVlcVideoEnabled condition;

    @BeforeEach
    void setUp() {
        lenient().when(context.getBeanFactory()).thenReturn(beanFactory);
        lenient().when(beanFactory.getBean(ApplicationArguments.class)).thenReturn(applicationArguments);
    }

    @Test
    void testConfigurationPhase_whenInvoked_shouldReturnRegisterBean() {
        var result = condition.getConfigurationPhase();

        assertEquals(ConfigurationCondition.ConfigurationPhase.REGISTER_BEAN, result);
    }

    @Test
    void testMatches_whenDisableVlcIsNotPresent_shouldReturnTrue() {
        when(applicationArguments.containsOption(OnVlcVideoEnabled.DISABLE_VLC_PLAYER)).thenReturn(false);

        var result = condition.matches(context, metadata);

        assertTrue(result, "Expected the condition to match");
    }

    @Test
    void testMatches_whenDisableVlcIsPresent_shouldReturnFalse() {
        when(applicationArguments.containsOption(OnVlcVideoEnabled.DISABLE_VLC_PLAYER)).thenReturn(true);

        var result = condition.matches(context, metadata);

        assertFalse(result, "Expected the condition be disabled");
    }
}