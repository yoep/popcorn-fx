package com.github.yoep.popcorn.ui.view.conditions;

import com.github.yoep.popcorn.backend.settings.OptionsService;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.beans.factory.config.ConfigurableListableBeanFactory;
import org.springframework.boot.ApplicationArguments;
import org.springframework.context.annotation.ConditionContext;
import org.springframework.core.type.AnnotatedTypeMetadata;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class OnTvModeConditionTest {
    @Mock
    private ConditionContext conditionContext;
    @Mock
    private AnnotatedTypeMetadata annotatedTypeMetadata;
    @Mock
    private ConfigurableListableBeanFactory beanFactory;
    @Mock
    private ApplicationArguments arguments;
    @InjectMocks
    private OnTvModeCondition condition;

    @Test
    void testMatches_whenNoBeanFactoryIsPresent_shouldReturnFalse() {
        var result = condition.matches(conditionContext, annotatedTypeMetadata);

        assertFalse(result, "Expected the condition to not match");
    }

    @Test
    void testMatches_whenTvModeIsEnabled_shouldReturnTrue() {
        when(conditionContext.getBeanFactory()).thenReturn(beanFactory);
        when(beanFactory.getBean(ApplicationArguments.class)).thenReturn(arguments);
        when(arguments.containsOption(OptionsService.TV_MODE_OPTION)).thenReturn(true);

        var result = condition.matches(conditionContext, annotatedTypeMetadata);

        assertTrue(result, "Expected the condition to match");
    }

    @Test
    void testMatches_whenTvModeIsDisabled_shouldReturnFalse() {
        when(conditionContext.getBeanFactory()).thenReturn(beanFactory);
        when(beanFactory.getBean(ApplicationArguments.class)).thenReturn(arguments);
        when(arguments.containsOption(OptionsService.TV_MODE_OPTION)).thenReturn(false);

        var result = condition.matches(conditionContext, annotatedTypeMetadata);

        assertFalse(result, "Expected the condition to not match");
    }
}
