package com.github.yoep.video.vlc.conditions;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.context.annotation.ConditionContext;
import org.springframework.context.annotation.ConfigurationCondition;
import org.springframework.core.type.AnnotatedTypeMetadata;

import static org.junit.jupiter.api.Assertions.*;

@ExtendWith(MockitoExtension.class)
class OnNonArmConditionTest {
    @Mock
    private ConditionContext context;
    @Mock
    private AnnotatedTypeMetadata metadata;
    @InjectMocks
    private OnNonArmCondition condition;

    @Test
    void testGetConfigurationPhase_whenInvoked_shouldReturnRegisterBean() {
        var expectedResult = ConfigurationCondition.ConfigurationPhase.REGISTER_BEAN;

        var result = condition.getConfigurationPhase();

        assertEquals(expectedResult, result);
    }

    @Test
    void testMatches_whenCpuIsNonArm_shouldReturnTrue() {
        System.setProperty("os.arch", "amd64");

        var result = condition.matches(context, metadata);

        assertTrue(result);
    }

    @Test
    void testMatches_whenCpuIsArm_shouldReturnFalse() {
        System.setProperty("os.arch", "arm");

        var result = condition.matches(context, metadata);

        assertFalse(result);
    }
}
